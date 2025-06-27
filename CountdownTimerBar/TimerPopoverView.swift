//
//  TimerPopoverView.swift
//  CountdownTimerBar
//
//  Created by Pavel Bubentsov on 27.06.2025.
//

import SwiftUI

struct TimerPopoverView: View {
    @State private var focusTimers = [15, 30, 35]
    @State private var restTimers = [1, 10, 20]
    @State private var showOptions = false
//    @StateObject private var timerModel = TimerModel()
    @EnvironmentObject var timerModel: TimerModel
    
    // Новый стейт: какой таймер сейчас активен (в секундах)
    private var activeValue: Int? {
        timerModel.isRunning ? timerModel.remainingSeconds : nil
    }

    var body: some View {
        VStack {
            // Показывать 00:00 если таймер не активен
            Text(formatTime(timerModel.isRunning ? timerModel.remainingSeconds : 0))
                .font(.largeTitle)
                .padding()
            HStack {
                VStack {
                    Text("Focus")
                    ForEach(focusTimers, id: \.self) { t in
                        let seconds = t * 60
                        TimerButton(
                            value: t,
                            isActive: activeValue == seconds,
                            action: {
                                if activeValue == seconds {
                                    timerModel.stop()
                                } else {
                                    timerModel.start(seconds: seconds)
                                }
                            }
                        )
                    }
                }
                VStack {
                    Text("Rest")
                    ForEach(restTimers, id: \.self) { t in
                        let seconds = t
                        TimerButton(
                            value: t,
                            isActive: activeValue == seconds,
                            action: {
                                if activeValue == seconds {
                                    timerModel.stop()
                                } else {
                                    timerModel.start(seconds: seconds)
                                }
                            }
                        )
                    }
                }
            }
            Button(action: { showOptions = true }) {
                Image(systemName: "gearshape")
            }
            .sheet(isPresented: $showOptions) {
                OptionsView(focusTimers: $focusTimers, restTimers: $restTimers)
            }
        }
        .padding()
        .frame(width: 250)
    }

    func formatTime(_ seconds: Int) -> String {
        let m = seconds / 60
        let s = seconds % 60
        return String(format: "%02d:%02d", m, s)
    }
}

struct TimerButton: View {
    let value: Int
    let isActive: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text("\(value)")
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
