//
//  TimerPopoverView.swift
//  CountdownTimerBar
//
//  Created by Pavel Bubentsov on 27.06.2025.
//

import SwiftUI

struct TimerPopoverView: View {
    @State private var focusTimers = [15, 30, 35]
    @State private var restTimers = [3, 10, 20]
    @State private var showOptions = false

    var body: some View {
        VStack {
            // Таймеры
            HStack {
                VStack {
                    Text("Focus")
                    ForEach(focusTimers, id: \.self) { t in
                        TimerButton(value: t)
                    }
                }
                VStack {
                    Text("Rest")
                    ForEach(restTimers, id: \.self) { t in
                        TimerButton(value: t)
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
}

struct TimerButton: View {
    let value: Int
    var body: some View {
        Text("\(value)")
            .frame(width: 60, height: 60)
            .background(RoundedRectangle(cornerRadius: 30).stroke(Color.purple, lineWidth: 3))
            .font(.title)
            .padding(4)
    }
}
