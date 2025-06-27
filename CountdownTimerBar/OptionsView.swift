//
//  OptionsView.swift
//  CountdownTimerBar
//
//  Created by Pavel Bubentsov on 27.06.2025.
//

import SwiftUI

struct OptionsView: View {
    @EnvironmentObject var settings: SettingsStore
    var onClose: (() -> Void)? = nil
    
    @State private var focusInput: String = ""
    @State private var restInput: String = ""
    @State private var showAbout = false

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Options").font(.headline)
            
            Toggle("Sound", isOn: $settings.soundOn)
            
            VStack(alignment: .leading) {
                Text("Focus Timers")
                TextField("e.g. 15, 30m, 45s", text: $focusInput, onCommit: {
                    settings.focusTimers = parseTimerInput(focusInput)
                })
            }
            
            VStack(alignment: .leading) {
                Text("Rest Timers")
                TextField("e.g. 1m, 5, 10s", text: $restInput, onCommit: {
                    settings.restTimers = parseTimerInput(restInput)
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
                        Text("Minimalist configurable timer for macOS StatusBar.\nInspired by Hourglass.")
                            .multilineTextAlignment(.center)
                        Text("© 2025 Pavel Bubentsov")
                            .font(.footnote)
                            .foregroundColor(.secondary)
                        Button("Close") { showAbout = false }
                    }
                    .padding()
                    .frame(width: 300)
                }
                Spacer()
                Button("Quit") {
                    DispatchQueue.main.async {
                        exit(0)
                    }
                }
            }
        }
        .padding()
        .frame(width: 300)
        .onAppear {
            self.focusInput = settings.focusTimers.map(formatForInput).joined(separator: ", ")
            self.restInput = settings.restTimers.map(formatForInput).joined(separator: ", ")
        }
    }

    private func formatForInput(_ seconds: Int) -> String {
        if seconds < 60 { return "\(seconds)s" }
        if seconds % 60 == 0 { return "\(seconds / 60)" }
        return "\(seconds)s"
    }

    private func parseTimerInput(_ input: String) -> [Int] {
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

