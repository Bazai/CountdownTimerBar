import SwiftUI
import Combine

class SettingsStore: ObservableObject {
    
    @Published var focusTimers: [Int] {
        didSet {
            UserDefaults.standard.set(focusTimers.map(String.init), forKey: "focusTimers")
        }
    }
    
    @Published var restTimers: [Int] {
        didSet {
            UserDefaults.standard.set(restTimers.map(String.init), forKey: "restTimers")
        }
    }
    
    @Published var soundOn: Bool {
        didSet {
            UserDefaults.standard.set(soundOn, forKey: "soundOn")
        }
    }
    
    init() {
        self.focusTimers = UserDefaults.standard.stringArray(forKey: "focusTimers")?.compactMap { Int($0) } ?? [900, 1800, 2100] // 15, 30, 35 min
        self.restTimers = UserDefaults.standard.stringArray(forKey: "restTimers")?.compactMap { Int($0) } ?? [60, 600, 1200] // 1, 10, 20 min
        self.soundOn = UserDefaults.standard.bool(forKey: "soundOn")
    }
} 
