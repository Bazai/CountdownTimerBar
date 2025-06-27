//
//  OptionsView.swift
//  CountdownTimerBar
//
//  Created by Pavel Bubentsov on 27.06.2025.
//

import SwiftUI

struct OptionsView: View {
    @Binding var focusTimers: [Int]
    @Binding var restTimers: [Int]
    @State private var focusInput = ""
    @State private var restInput = ""
    @State private var soundOn = true
    @State private var checkboxOption = false
    @State private var showAbout = false

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Options").font(.headline)
            Toggle("Sound", isOn: $soundOn)
            Toggle("Checkbox Option", isOn: $checkboxOption)
            VStack(alignment: .leading) {
                Text("Focus Timers")
                TextField("10,15,30", text: $focusInput, onCommit: {
                    focusTimers = parseTimerInput(focusInput)
                })
            }
            VStack(alignment: .leading) {
                Text("Rest Timers")
                TextField("2,3,5,19", text: $restInput, onCommit: {
                    restTimers = parseTimerInput(restInput)
                })
            }
            HStack {
                Button("About") {
                    showAbout = true
                }
                .sheet(isPresented: $showAbout) {
                    VStack(spacing: 20) {
                        Text("CountdownTimerBar")
                            .font(.title)
                        Text("Minimalist configurable timer for macOS StatusBar.\nInspired by Hourglass UI.")
                            .multilineTextAlignment(.center)
                        Button("Close") { showAbout = false }
                    }
                    .padding()
                    .frame(width: 300)
                }
                Spacer()
                Button("Quit") {
                    NSApp.terminate(nil)
                }
            }
        }
        .padding()
        .frame(width: 300)
        .onAppear {
            focusInput = focusTimers.map { $0 % 60 == 0 ? "\($0/60)" : "\($0)s" }.joined(separator: ",")
            restInput = restTimers.map { $0 % 60 == 0 ? "\($0/60)" : "\($0)s" }.joined(separator: ",")
        }
    }

    func parseTimerInput(_ input: String) -> [Int] {
        input.split(separator: ",").compactMap { part in
            let trimmed = part.trimmingCharacters(in: .whitespaces)
            if trimmed.hasSuffix("s") {
                return Int(trimmed.dropLast())
            } else if trimmed.hasSuffix("m") {
                if let min = Int(trimmed.dropLast()) { return min * 60 }
            } else if let min = Int(trimmed) {
                return min * 60
            }
            return nil
        }
    }
}
