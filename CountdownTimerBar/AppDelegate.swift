//
//  AppDelegate.swift
//  CountdownTimerBar
//
//  Created by Pavel Bubentsov on 27.06.2025.
//

import Cocoa
import Combine
import SwiftUI

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
                if self?.timerModel.isRunning == true {
                    self?.statusItem?.button?.title =
                        self?.formatTime(seconds) ?? ""
                } else {
                    self?.statusItem?.button?.title = ""
                    self?.statusItem?.button?.image = NSImage(
                        systemSymbolName: "hourglass",
                        accessibilityDescription: "Timer"
                    )
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
}
