import SwiftUI

@main
struct FerroApp: App {
    @State private var appState = AppState.shared

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(appState)
        }
    }
}
