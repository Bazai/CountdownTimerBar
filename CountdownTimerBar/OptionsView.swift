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

    @State private var focusInputError = false
    @State private var restInputError = false

    private enum InputField { case focus, rest }

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            Text("Options").font(.headline)
            
            Toggle("Sound", isOn: $settings.soundOn)
            
            VStack(alignment: .leading) {
                Text("Focus Timers")
                TextField("e.g. 15, 30m, 45s", text: $focusInput, onCommit: {
                    let result = parseTimerInput(focusInput)
                    settings.focusTimers = result.values
                    if result.hasInvalid {
                        triggerErrorAnimation(for: .focus)
                    }
                })
                .overlay(
                    RoundedRectangle(cornerRadius: 4)
                        .stroke(focusInputError ? Color.red : Color.clear, lineWidth: 1.5)
                )
            }
            
            VStack(alignment: .leading) {
                Text("Rest Timers")
                TextField("e.g. 1m, 5, 10s", text: $restInput, onCommit: {
                    let result = parseTimerInput(restInput)
                    settings.restTimers = result.values
                    if result.hasInvalid {
                        triggerErrorAnimation(for: .rest)
                    }
                })
                .overlay(
                    RoundedRectangle(cornerRadius: 4)
                        .stroke(restInputError ? Color.red : Color.clear, lineWidth: 1.5)
                )
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

    private func triggerErrorAnimation(for field: InputField) {
        let errorStateBinding: Binding<Bool> = (field == .focus) ? $focusInputError : $restInputError
        
        withAnimation { errorStateBinding.wrappedValue = true }
        
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
            withAnimation { errorStateBinding.wrappedValue = false }
        }
    }

    private func formatForInput(_ seconds: Int) -> String {
        if seconds < 60 { return "\(seconds)s" }
        if seconds % 60 == 0 { return "\(seconds / 60)" }
        return "\(seconds)s"
    }

    private func parseTimerInput(_ input: String) -> (values: [Int], hasInvalid: Bool) {
        var values: [Int] = []
        var hasInvalidInput = false
        
        if input.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            return ([], false)
        }
        
        let parts = input.split(separator: ",")
        for part in parts {
            let trimmed = part.trimmingCharacters(in: .whitespaces)
            guard !trimmed.isEmpty else { continue }
            
            var parsedValue: Int? = nil
            
            if trimmed.hasSuffix("s") {
                parsedValue = Int(trimmed.dropLast())
            } else if trimmed.hasSuffix("m") {
                if let minutes = Int(trimmed.dropLast()) {
                    parsedValue = minutes * 60
                }
            } else if let minutes = Int(trimmed) {
                parsedValue = minutes * 60
            }
            
            if let value = parsedValue {
                values.append(value)
            } else {
                hasInvalidInput = true
            }
        }
        return (values, hasInvalidInput)
    }
}

