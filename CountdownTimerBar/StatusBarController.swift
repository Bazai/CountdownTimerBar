import Cocoa
import Combine
import SwiftUI

class StatusBarController {
    private var statusItem: NSStatusItem
    private let popover: NSPopover
    private let timerModel: TimerModel
    private let settingsStore: SettingsStore
    private var cancellable: AnyCancellable?

    init() {
        self.statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        self.popover = NSPopover()
        self.settingsStore = SettingsStore()
        self.timerModel = TimerModel(settings: self.settingsStore)
        
        setupStatusItem()
        setupPopover()
        setupTimerSubscription()
    }

    @objc private func togglePopover(_ sender: AnyObject?) {
        if popover.isShown {
            popover.performClose(sender)
        } else if let button = statusItem.button {
            popover.show(
                relativeTo: button.bounds,
                of: button,
                preferredEdge: .minY
            )
        }
    }
    
    private func setupStatusItem() {
        if let button = statusItem.button {
            button.action = #selector(togglePopover(_:))
            button.target = self
            button.image = makePillImage(text: "00:00")
            button.imagePosition = .imageOnly
        }
    }
    
    private func setupPopover() {
        let timerView = TimerPopoverView()
            .environmentObject(timerModel)
            .environmentObject(settingsStore)
        
        popover.contentViewController = NSHostingController(rootView: timerView)
        popover.behavior = .transient
    }
    
    private func setupTimerSubscription() {
        cancellable = timerModel.$remainingSeconds.sink { [weak self] seconds in
            DispatchQueue.main.async {
                guard let self = self else { return }
                let time = self.formatTime(self.timerModel.isRunning ? seconds : 0)
                self.statusItem.button?.image = self.makePillImage(text: time)
            }
        }
    }

    private func formatTime(_ seconds: Int) -> String {
        let minutes = seconds / 60
        let seconds = seconds % 60
        return String(format: "%02d:%02d", minutes, seconds)
    }

    private func makePillImage(text: String) -> NSImage {
        let font = NSFont.systemFont(ofSize: 13, weight: .medium)
        let attributes: [NSAttributedString.Key: Any] = [
            .font: font,
            .foregroundColor: NSColor.labelColor
        ]
        let size = (text as NSString).size(withAttributes: attributes)
        let pillHeight = ceil(size.height + 3)
        let pillWidth = ceil(size.width + 9)
        let imageSize = NSSize(width: pillWidth, height: pillHeight)
        
        let image = NSImage(size: imageSize)
        image.lockFocus()
        
        if let ctx = NSGraphicsContext.current {
            ctx.shouldAntialias = true
            ctx.imageInterpolation = .high
        }
        
        let rect = NSRect(x: 0.5, y: 0.5, width: pillWidth - 1, height: pillHeight - 1)
        let path = NSBezierPath(roundedRect: rect, xRadius: pillHeight / 2, yRadius: pillHeight / 2)
        
        NSColor.clear.setFill()
        path.fill()
        
        NSColor.labelColor.setStroke()
        path.lineWidth = 1.0
        path.stroke()
        
        let textRect = NSRect(
            x: (pillWidth - size.width) / 2,
            y: (pillHeight - size.height) / 2,
            width: size.width,
            height: size.height
        )
        (text as NSString).draw(in: textRect, withAttributes: attributes)
        
        image.unlockFocus()
        return image
    }
} 
