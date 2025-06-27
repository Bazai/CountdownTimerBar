//
//  AppDelegate.swift
//  CountdownTimerBar
//
//  Created by Pavel Bubentsov on 27.06.2025.
//

import Cocoa
import Combine
import SwiftUI
import AppKit

class AppDelegate: NSObject, NSApplicationDelegate {
    var statusItem: NSStatusItem?
    var popover = NSPopover()
    var timerModel = TimerModel()
    var cancellable: AnyCancellable?

    func applicationDidFinishLaunching(_ notification: Notification) {
        statusItem = NSStatusBar.system.statusItem(
            withLength: NSStatusItem.variableLength
        )
        if let button = statusItem?.button {
            button.action = #selector(togglePopover(_:))
            button.target = self
            button.image = NSImage(
                systemSymbolName: "hourglass",
                accessibilityDescription: "Timer"
            )
        }
        popover.contentViewController = NSHostingController(
            rootView: TimerPopoverView().environmentObject(timerModel)
        )

        // Подписка на обновление времени
        cancellable = timerModel.$remainingSeconds.sink { [weak self] seconds in
            DispatchQueue.main.async {
                if let button = self?.statusItem?.button {
                    let time = self?.formatTime(self?.timerModel.isRunning == true ? seconds : 0) ?? "00:00"
                    button.image = self?.makePillImage(text: time)
                    button.imagePosition = .imageOnly
                    button.title = ""
                }
            }
        }
    }
    
    func formatTime(_ seconds: Int) -> String {
        let m = seconds / 60
        let s = seconds % 60
        return String(format: "%02d:%02d", m, s)
    }

    @objc func togglePopover(_ sender: AnyObject?) {
        if popover.isShown {
            popover.performClose(sender)
        } else if let button = statusItem?.button {
            popover.show(
                relativeTo: button.bounds,
                of: button,
                preferredEdge: .minY
            )
        }
    }

    func makePillImage(text: String) -> NSImage {
        let font = NSFont.systemFont(ofSize: 13, weight: .medium)
        let attributes: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: NSColor.labelColor
        ]
        let size = (text as NSString).size(withAttributes: attributes)
        let pillHeight: CGFloat = ceil(size.height + 3)
        let pillWidth: CGFloat = ceil(size.width + 9)
        let imageSize = NSSize(width: pillWidth, height: pillHeight)
        _ = NSScreen.main?.backingScaleFactor ?? 2.0
        let image = NSImage(size: imageSize)
        image.lockFocusFlipped(false)
        if let ctx = NSGraphicsContext.current {
            ctx.shouldAntialias = true
            ctx.imageInterpolation = .high
        }
        let rect = NSRect(x: 0.5, y: 0.5, width: pillWidth-1, height: pillHeight-1)
        let path = NSBezierPath(roundedRect: rect, xRadius: pillHeight/2, yRadius: pillHeight/2)
        NSColor.clear.setFill()
        path.fill()
        NSColor.labelColor.setStroke()
        path.lineWidth = 1.0
        path.stroke()
        // Центрируем текст
        let textRect = NSRect(
            x: (pillWidth - size.width)/2,
            y: (pillHeight - size.height)/2,
            width: size.width,
            height: size.height
        )
        (text as NSString).draw(in: textRect, withAttributes: attributes)
        image.unlockFocus()
        image.size = imageSize
        return image
    }
}
