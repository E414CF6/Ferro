import SwiftUI

struct ContentView: View {
    @Environment(AppState.self) private var appState

    var body: some View {
        Group {
            if appState.isAuthenticated {
                TabView(selection: Bindable(appState).selectedTab) {
                    FeedView()
                        .tabItem {
                            Label(MainTab.feed.title, systemImage: MainTab.feed.systemImage)
                        }
                        .tag(MainTab.feed)

                    ExploreView()
                        .tabItem {
                            Label(MainTab.explore.title, systemImage: MainTab.explore.systemImage)
                        }
                        .tag(MainTab.explore)

                    ConversationsView()
                        .tabItem {
                            Label(MainTab.messages.title, systemImage: MainTab.messages.systemImage)
                        }
                        .tag(MainTab.messages)
                        .badge(appState.unreadMessagesCount)

                    NotificationsView()
                        .tabItem {
                            Label(MainTab.notifications.title, systemImage: MainTab.notifications.systemImage)
                        }
                        .tag(MainTab.notifications)
                        .badge(appState.unreadNotificationsCount)

                    ProfileView()
                        .tabItem {
                            Label(MainTab.profile.title, systemImage: MainTab.profile.systemImage)
                        }
                        .tag(MainTab.profile)
                }
                .tint(.indigo)
            } else {
                AuthView()
            }
        }
        .alert("Notice", isPresented: Bindable(appState).isShowingError) {
            Button("OK", role: .cancel) {}
        } message: {
            if let err = appState.errorMessage {
                Text(err)
            }
        }
    }
}

#Preview {
    ContentView()
        .environment(AppState.shared)
}
