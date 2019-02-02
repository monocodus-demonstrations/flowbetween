//
//  FloCanvasLayer.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 25/01/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// Layer that renders a canvas
///
class FloCanvasLayer : CALayer {
    /// The backing for this layer (nil if it's not drawable yet)
    var _backing: [UInt32: CGLayer];
    
    /// Layers that we stopped using during the last clear command
    var _unusedLayers: [CGLayer];
    
    /// Function called to trigger a redraw
    var _triggerRedraw: ((NSSize, NSRect) -> ())?;
    
    /// The overall size of the canvas
    var _canvasSize: NSSize;
    
    /// The coordinates of the visible region in the canvsa
    var _visibleRect: NSRect;
    
    /// The resolution of this layer
    var _resolution: CGFloat = 1.0;
    
    override init() {
        _canvasSize     = NSSize(width: 1, height: 1);
        _visibleRect    = NSRect(x: 0, y: 0, width: 1, height: 1);
        _backing        = [UInt32: CGLayer]();
        _unusedLayers   = [];

        super.init();
    }
    
    override init(layer: Any) {
        _canvasSize     = NSSize(width: 1, height: 1);
        _visibleRect    = NSRect(x: 0, y: 0, width: 1, height: 1);
        _backing        = [UInt32: CGLayer]();
        _unusedLayers   = [];

        super.init();
        
        if let layer = layer as? FloCanvasLayer {
            _backing            = layer._backing;
            _canvasSize         = layer._canvasSize;
            _visibleRect        = layer._visibleRect;
            _resolution         = layer._resolution;
        }
    }
    
    required init?(coder aDecoder: NSCoder) {
        _canvasSize     = NSSize(width: 1, height: 1);
        _visibleRect    = NSRect(x: 0, y: 0, width: 1, height: 1);
        _backing        = [UInt32: CGLayer]();
        _unusedLayers   = [];

        super.init(coder: aDecoder);
    }
    
    override func draw(in ctx: CGContext) {
        // Redraw the backing layer if it has been invalidated
        if _backing.count == 0 {
            var size    = _visibleRect.size;
            size.width  *= _resolution;
            size.height *= _resolution;
            
            if size.width == 0 { size.width = 1; }
            if size.height == 0 { size.height = 1; }
            
            // Create the backing layer (there's always a layer 0 by default)
            _backing[0] = CGLayer(ctx, size: size, auxiliaryInfo: nil);
            
            if _resolution != 1.0 {
                let scale = CGAffineTransform.init(scaleX: _resolution, y: _resolution);
                _backing[0]!.context!.concatenate(scale);
            }
            
            // Force a redraw via the events
            _triggerRedraw?(_canvasSize, _visibleRect);
        }
        
        // Draw the backing layer on this layer
        let layer_ids   = _backing.keys.sorted();
        let bounds      = self.bounds;
        
        ctx.saveGState();
        ctx.setShouldAntialias(false);
        ctx.interpolationQuality = CGInterpolationQuality.none;
        if _resolution != 1.0 {
            ctx.concatenate(CGAffineTransform.init(scaleX: 1.0/_resolution, y: 1.0/_resolution));
        }
        
        for layer_id in layer_ids {
            ctx.draw(_backing[layer_id]!, at: bounds.origin);
        }
        
        ctx.restoreGState();
    }
    
    ///
    /// Clears the backing layers for this layer
    ///
    func clearBackingLayers() {
        // All layers other than layer 0 are removed (pushed onto the unused layer list)
        let layers_to_remove = _backing.keys.filter({ layer_id in layer_id != 0 });
        for layer_id in layers_to_remove {
            _unusedLayers.append(_backing[layer_id]!);
            _backing.removeValue(forKey: layer_id);
        }
        
        // Clear the bottom layer
        _backing[0]?.context?.clear(CGRect(origin: CGPoint(x: 0, y: 0), size: self.bounds.size));
    }
    
    ///
    /// Ensures the layer with the specifed ID exists
    ///
    func getContextForLayer(id: UInt32) -> CGContext? {
        if _backing.keys.contains(id) {
            // Layer already exists
            return _backing[id]?.context;
        } else if let availableLayer = _unusedLayers.popLast() {
            // Use a layer we created earlier if we can
            _backing[id] = availableLayer;
            
            // Make sure it has nothing already rendered on it
            availableLayer.context?.clear(CGRect(origin: CGPoint.zero, size: _visibleRect.size));
            return availableLayer.context;
        } else if let baseLayer = _backing[0] {
            // Get the size for the new layer
            var size    = _visibleRect.size;
            size.width  *= _resolution;
            size.height *= _resolution;
            
            if size.width == 0 { size.width = 1; }
            if size.height == 0 { size.height = 1; }

            // We create the new layer from a base layer (as CGLayer needs a context to work from)
            let newLayer = CGLayer(baseLayer.context!, size: size, auxiliaryInfo: nil);
            
            if _resolution != 1.0 {
                let scale = CGAffineTransform.init(scaleX: _resolution, y: _resolution);
                newLayer!.context!.concatenate(scale);
            }
            
            // Store the new layer as a new backing layer
            _backing[id] = newLayer!;
            return newLayer?.context;
        } else {
            // No base layer, so we can't create new layers
            return nil;
        }
    }
    
    ///
    /// Invalidates all of the layers in this object.
    ///
    /// This will remove them entirely: normally when the canvas is cleared we keep track of
    /// any layers we were using before so we don't need to reallocate them in the event of
    /// a redraw. However, this will produce invalid results when the layer is resized.
    ///
    func invalidateAllLayers() {
        // Both the backing and the unused layers become invalidated so we can't re-use them
        _backing        = [UInt32: CGLayer]();
        _unusedLayers   = [];
    }
}
