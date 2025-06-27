//
//  TimerModel.swift
//  CountdownTimerBar
//
//  Created by Pavel Bubentsov on 27.06.2025.
//

import Foundation
import Combine
import AppKit
import UserNotifications

class TimerModel: ObservableObject {
    @Published var remainingSeconds: Int = 0
    @Published var isRunning: Bool = false

    private var timer: Timer?

    func start(seconds: Int) {
        remainingSeconds = seconds
        isRunning = true
        timer?.invalidate()
        timer = Timer.scheduledTimer(withTimeInterval: 1, repeats: true) { [weak self] _ in
            self?.tick()
        }
    }

    func stop() {
        timer?.invalidate()
        timer = nil
        isRunning = false
        remainingSeconds = 0
    }

    private func tick() {
        if remainingSeconds > 1 {
            remainingSeconds -= 1
        } else {
            remainingSeconds = 0
            isRunning = false
            timer?.invalidate()
            timer = nil
            // Уведомление через UserNotifications
            let center = UNUserNotificationCenter.current()
            center.requestAuthorization(options: [.alert, .sound]) { granted, _ in
                if granted {
                    let content = UNMutableNotificationContent()
                    content.title = "Timer finished"
                    content.body = "Countdown complete."
                    if UserDefaults.standard.bool(forKey: "soundOn") {
                        content.sound = UNNotificationSound.default
                    }
                    let request = UNNotificationRequest(identifier: UUID().uuidString, content: content, trigger: nil)
                    center.add(request, withCompletionHandler: nil)
                }
            }
            // Звук (если включено)
            if UserDefaults.standard.bool(forKey: "soundOn") {
                NSSound(named: NSSound.Name("Glass"))?.play()
            }
        }
    }
}
