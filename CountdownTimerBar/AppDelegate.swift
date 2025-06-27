//
//  AppDelegate.swift
//  CountdownTimerBar
//
//  Created by Pavel Bubentsov on 27.06.2025.
//

import Cocoa
import SwiftUI

class AppDelegate: NSObject, NSApplicationDelegate {
    var statusItem: NSStatusItem?
    var popover = NSPopover()

    func applicationDidFinishLaunching(_ notification: Notification) {
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        if let button = statusItem?.button {
            button.action = #selector(togglePopover(_:))
            button.target = self
            button.image = NSImage(systemSymbolName: "hourglass", accessibilityDescription: "Timer")
        }
        popover.contentViewController = NSHostingController(rootView: TimerPopoverView())
    }

    @objc func togglePopover(_ sender: AnyObject?) {
        if popover.isShown {
            popover.performClose(sender)
        } else if let button = statusItem?.button {
            popover.show(relativeTo: button.bounds, of: button, preferredEdge: .minY)
        }
    }
}
