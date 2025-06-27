//
//  TimerModel.swift
//  CountdownTimerBar
//
//  Created by Pavel Bubentsov on 27.06.2025.
//

import Foundation
import Combine

class TimerModel: ObservableObject {
    @Published var remainingSeconds: Int = 0
    @Published var isRunning: Bool = false

    private var timer: Timer?
    private var endDate: Date?

    func start(seconds: Int) {
        remainingSeconds = seconds
        isRunning = true
        endDate = Date().addingTimeInterval(TimeInterval(seconds))
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
        endDate = nil
    }

    private func tick() {
        guard let endDate else { return }
        let left = Int(endDate.timeIntervalSinceNow)
        if left > 0 {
            remainingSeconds = left
        } else {
            remainingSeconds = 0
            isRunning = false
            timer?.invalidate()
            timer = nil
            // Можно добавить уведомление или звук тут
        }
    }
}
