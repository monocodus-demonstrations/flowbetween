//
//  FloContainerButtonLayer.swift
//  FlowBetween
//
//  Created by Andrew Hunter on 09/02/2019.
//  Copyright © 2019 Andrew Hunter. All rights reserved.
//

import Cocoa

///
/// A layer representing the background of a conainer button
///
class FloContainerButtonLayer : CALayer {
    fileprivate var _highlighted        = false;
    fileprivate var _selected           = false;
    fileprivate var _classes: [String]  = [];
    fileprivate var _isFirst: Bool      = false;
    fileprivate var _isLast: Bool       = false;
    
    /// Set to true if this button is highlighted
    var highlighted: Bool {
        get { return _highlighted; }
        set(value) { _highlighted = value; setNeedsDisplay(); }
    }
    
    /// Set to true if this button is selected
    var selected: Bool {
        get { return _selected; }
        set(value) { _selected = value; setNeedsDisplay(); }
    }
    
    /// True if this button is first in the list
    var isFirst: Bool {
        get { return _isFirst; }
        set(value) { _isFirst = value; setNeedsDisplay(); }
    }
    
    /// True if this button is last in the list
    var isLast: Bool {
        get { return _isLast; }
        set(value) { _isLast = value; setNeedsDisplay(); }
    }
    
    /// The classes that should be applied to this button
    var classes: [String] {
        get { return _classes; }
        set(value) { _classes = value; setNeedsDisplay(); }
    }
    
    override func resize(withOldSuperlayerSize size: CGSize) {
        setNeedsDisplay();
        super.resize(withOldSuperlayerSize: size);
    }
    
    /// Draws the content of this layer
    override func draw(in ctxt: CGContext) {
        let isInGroup = classes.contains("button-group");
        
        if isInGroup {
            drawInButtonGroup(in: ctxt);
        } else {
            drawWithDefaultClass(in: ctxt);
        }
    }
    
    /// Draws the content of this layer in the default style
    func drawWithDefaultClass(in ctxt: CGContext) {
        let background: CGColor;
        let border:     CGColor;
        
        // Colours are based on whether or not we're highlighted or selected
        if highlighted && selected {
            border      = CGColor.init(red: 0.5, green: 0.6, blue: 0.7, alpha: 1.0);
            background  = CGColor.init(red: 0.63, green: 0.59, blue: 0.78, alpha: 0.8);
        } else if selected {
            border      = CGColor.init(red: 0.59, green: 0.55, blue: 0.86, alpha: 0.8);
            background  = CGColor.init(red: 0.0, green: 0.0, blue: 0.0, alpha: 0.8);
        } else if highlighted {
            border      = CGColor.clear;
            background  = CGColor.init(red: 0.7, green: 0.7, blue: 0.8, alpha: 0.5);
        } else {
            border      = CGColor.clear;
            background  = CGColor.init(red: 0.4, green: 0.4, blue: 0.4, alpha: 0.2);
        }
        
        // Draw the button background
        let rounded = CGPath.init(roundedRect: bounds.insetBy(dx: 2.0, dy: 2.0), cornerWidth: 6.0, cornerHeight: 6.0, transform: nil);
        ctxt.beginPath();
        ctxt.addPath(rounded);
        ctxt.setFillColor(background);
        ctxt.fillPath();

        // And the border
        ctxt.beginPath();
        ctxt.addPath(rounded);
        ctxt.setStrokeColor(border);
        ctxt.setLineWidth(1.5);
        ctxt.strokePath();
    }
    
    /// Draws the content of this layer (when it's part of a button group)
    func drawInButtonGroup(in ctxt: CGContext) {
        let bounds              = self.bounds;
        let isFirst             = self.isFirst;
        let isLast              = self.isLast;
        let background: CGColor;
        let border:     CGColor;
        
        // Colours are based on whether or not we're highlighted or selected
        if highlighted && selected {
            border      = CGColor.init(red: 0.5, green: 0.6, blue: 0.7, alpha: 1.0);
            background  = CGColor.init(red: 0.63, green: 0.59, blue: 0.78, alpha: 0.8);
        } else if selected {
            border      = CGColor.init(red: 0.59, green: 0.55, blue: 0.86, alpha: 0.8);
            background  = CGColor.init(red: 0.0, green: 0.0, blue: 0.0, alpha: 0.8);
        } else if highlighted {
            border      = CGColor.clear;
            background  = CGColor.init(red: 0.7, green: 0.7, blue: 0.8, alpha: 0.5);
        } else {
            border      = CGColor.init(red: 0.63, green: 0.69, blue: 0.69, alpha: 1.0);
            background  = CGColor.init(red: 0.4, green: 0.4, blue: 0.4, alpha: 0.2);
        }
        
        // Generate the button path
        let rounded     = CGMutablePath.init();
        let cornerWidth = CGFloat.minimum(bounds.size.width/2.0, bounds.size.height/2.0);
        
        // Bottom
        if isLast {
            rounded.move(to: CGPoint(x: bounds.maxX-cornerWidth-1, y: bounds.minY));
        } else {
            rounded.move(to: CGPoint(x: bounds.maxX, y: bounds.minY));
        }
        
        if isFirst {
            rounded.addLine(to: CGPoint(x: bounds.minX+cornerWidth+1, y: bounds.minY));
        } else {
            rounded.addLine(to: CGPoint(x: bounds.minX, y: bounds.minY));
        }

        // LHS
        if isFirst {
            rounded.addArc(center: CGPoint(x: bounds.minX+cornerWidth+1, y: bounds.minY+cornerWidth),
                           radius: cornerWidth, startAngle: 3*CGFloat.pi/2, endAngle: 2*CGFloat.pi/2, clockwise: true);
            rounded.addLine(to: CGPoint(x: bounds.minX+1, y: bounds.maxY - cornerWidth));
            rounded.addArc(center: CGPoint(x: bounds.minX+cornerWidth+1, y: bounds.maxY-cornerWidth),
                           radius: cornerWidth, startAngle: 2*CGFloat.pi/2, endAngle: 1*CGFloat.pi/2, clockwise: true);
        } else {
            rounded.addLine(to: CGPoint(x: bounds.minX, y: bounds.maxY));
        }
        
        // Top
        if isLast {
            rounded.addLine(to: CGPoint(x: bounds.maxX-cornerWidth-1, y: bounds.maxY));

            rounded.addArc(center: CGPoint(x: bounds.maxX-cornerWidth-1, y: bounds.maxY-cornerWidth),
                           radius: cornerWidth, startAngle: 1*CGFloat.pi/2, endAngle: 0*CGFloat.pi/2, clockwise: true);
            rounded.addLine(to: CGPoint(x: bounds.maxX-1, y: bounds.minY + cornerWidth));
            rounded.addArc(center: CGPoint(x: bounds.maxX-cornerWidth-1, y: bounds.minY+cornerWidth),
                           radius: cornerWidth, startAngle: 0*CGFloat.pi/2, endAngle: -1*CGFloat.pi/2, clockwise: true);
        } else {
            rounded.addLine(to: CGPoint(x: bounds.maxX, y: bounds.maxY));
        }

        // Draw the button background
        ctxt.beginPath();
        ctxt.addPath(rounded);
        ctxt.setFillColor(background);
        ctxt.fillPath();
        
        // And the border
        ctxt.beginPath();
        ctxt.addPath(rounded);
        ctxt.setStrokeColor(border);
        ctxt.setLineWidth(1.5);
        ctxt.strokePath();
    }
}
