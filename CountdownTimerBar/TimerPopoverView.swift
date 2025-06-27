//
//  TimerPopoverView.swift
//  CountdownTimerBar
//
//  Created by Pavel Bubentsov on 27.06.2025.
//

import SwiftUI

enum TimerType: String, CaseIterable {
    case focus = "Focus"
    case rest = "Rest"
}

struct TimerSectionView: View {
    let type: TimerType
    @Binding var activeTimer: (type: TimerType, index: Int)?
    @EnvironmentObject var settings: SettingsStore
    @EnvironmentObject var timerModel: TimerModel

    private var timers: [Int] {
        switch type {
        case .focus: return settings.focusTimers
        case .rest: return settings.restTimers
        }
    }
    
    var body: some View {
        VStack(alignment: .center, spacing: 8) {
            Text(type.rawValue)
            ForEach(Array(timers.enumerated()), id: \.offset) { index, seconds in
                TimerButton(
                    value: seconds,
                    isActive: activeTimer?.type == type && activeTimer?.index == index && timerModel.isRunning,
                    display: displayString(for: seconds)
                ) {
                    if activeTimer?.type == type && activeTimer?.index == index && timerModel.isRunning {
                        timerModel.stop()
                        activeTimer = nil
                    } else {
                        timerModel.start(seconds: seconds)
                        activeTimer = (type, index)
                    }
                }
            }
        }
    }
    
    private func displayString(for seconds: Int) -> String {
        if seconds < 60 { return "\(seconds)s" }
        if seconds % 60 == 0 { return "\(seconds / 60)" }
        return "\(seconds)s"
    }
}

struct TimerPopoverView: View {
    @EnvironmentObject var settings: SettingsStore
    @EnvironmentObject var timerModel: TimerModel
    
    @State private var showOptions = false
    @State private var activeTimer: (type: TimerType, index: Int)? = nil

    var body: some View {
        VStack {
            Text(formatTime(timerModel.isRunning ? timerModel.remainingSeconds : 0))
                .font(.largeTitle)
                .padding()
            
            HStack(alignment: .top) {
                ForEach(TimerType.allCases, id: \.self) { timerType in
                    TimerSectionView(type: timerType, activeTimer: $activeTimer)
                }
            }
            
            Button(action: { showOptions = true }) {
                Image(systemName: "gearshape")
            }
            .popover(isPresented: $showOptions, arrowEdge: .bottom) {
                OptionsView(onClose: { showOptions = false })
            }
        }
        .padding()
        .frame(width: 250)
    }

    private func formatTime(_ seconds: Int) -> String {
        let minutes = seconds / 60
        let seconds = seconds % 60
        return String(format: "%02d:%02d", minutes, seconds)
    }
}

struct TimerButton: View {
    let value: Int
    let isActive: Bool
    let display: String
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text(display)
                .foregroundColor(Color.black)
                .frame(width: 60, height: 60)
                .background(
                    RoundedRectangle(cornerRadius: 30)
                        .fill(Color.white)
                )
                .overlay(
                    RoundedRectangle(cornerRadius: 30)
                        .stroke(isActive ? Color.purple : Color.clear, lineWidth: 4)
                )
                .font(.title)
                .padding(4)
        }
        .buttonStyle(PlainButtonStyle())
    }
}
