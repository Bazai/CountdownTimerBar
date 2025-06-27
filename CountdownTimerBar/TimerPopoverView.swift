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

    var body: some View {
        VStack {
            // Таймер в статус-баре
            if timerModel.isRunning {
                Text(formatTime(timerModel.remainingSeconds))
                    .font(.largeTitle)
                    .padding()
            }

            HStack {
                VStack {
                    Text("Focus")
                    ForEach(focusTimers, id: \.self) { t in
                        TimerButton(value: t) {
                            timerModel.start(seconds: t * 60)
                        }
                    }
                }
                VStack {
                    Text("Rest")
                    ForEach(restTimers, id: \.self) { t in
                        TimerButton(value: t) {
                            timerModel.start(seconds: t * 60)
                        }
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
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Text("\(value)")
                .frame(width: 60, height: 60)
                .background(
                    RoundedRectangle(cornerRadius: 30).stroke(
                        Color.purple,
                        lineWidth: 3
                    )
                )
                .font(.title)
                .padding(4)
        }
    }
}
