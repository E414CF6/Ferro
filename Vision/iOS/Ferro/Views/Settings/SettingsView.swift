import SwiftUI

public struct SettingsView: View {
    @Environment(AppState.self) private var appState
    @Environment(\.dismiss) private var dismiss

    @State private var baseURLText: String = APIConfiguration.shared.baseURLString
    @State private var isProbing = false
    @State private var probeResult: String? = nil

    public init() {}

    public var body: some View {
        NavigationStack {
            Form {
                // Section 1: Server Configuration
                Section("Serve API Backend Endpoint") {
                    #if os(iOS)
                    HStack {
                        Text("Base URL")
                        Spacer()
                        TextField("http://localhost:8080", text: $baseURLText)
                            .textInputAutocapitalization(.never)
                            .autocorrectionDisabled()
                            .multilineTextAlignment(.trailing)
                    }
                    #else
                    HStack {
                        Text("Base URL")
                        Spacer()
                        TextField("http://localhost:8080", text: $baseURLText)
                            .autocorrectionDisabled()
                            .multilineTextAlignment(.trailing)
                    }
                    #endif

                    Button("Save & Apply URL") {
                        APIConfiguration.shared.baseURLString = baseURLText
                        probeServerNow()
                    }

                    // Presets
                    Menu("Preset Servers") {
                        Button("Localhost (http://localhost:8080)") {
                            baseURLText = "http://localhost:8080"
                            APIConfiguration.shared.baseURLString = baseURLText
                            probeServerNow()
                        }
                        Button("127.0.0.1 (http://127.0.0.1:8080)") {
                            baseURLText = "http://127.0.0.1:8080"
                            APIConfiguration.shared.baseURLString = baseURLText
                            probeServerNow()
                        }
                    }

                    Button("Reset to Default") {
                        APIConfiguration.shared.resetToDefault()
                        baseURLText = APIConfiguration.shared.baseURLString
                        probeServerNow()
                    }
                    .foregroundStyle(.secondary)
                }

                // Section 2: Server Diagnostics Probe
                Section("Health & Diagnostics Probes") {
                    HStack {
                        Text("Liveness Status")
                        Spacer()
                        HStack(spacing: 6) {
                            Circle()
                                .fill(appState.isServerLive ? Color.green : Color.red)
                                .frame(width: 8, height: 8)
                            Text(appState.isServerLive ? "Live (/health/live)" : "Offline")
                                .font(.footnote)
                                .foregroundStyle(.secondary)
                        }
                    }

                    HStack {
                        Text("Readiness")
                        Spacer()
                        Text(appState.isServerReady ? "Ready (Postgres + Redis)" : "Not Ready")
                            .font(.footnote)
                            .foregroundStyle(appState.isServerReady ? .green : .secondary)
                    }

                    if let result = probeResult {
                        Text(result)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }

                    Button {
                        probeServerNow()
                    } label: {
                        HStack {
                            if isProbing {
                                ProgressView()
                            } else {
                                Label("Run Health Probe (/health/ready)", systemImage: "waveform.path.ecg")
                            }
                        }
                    }
                    .disabled(isProbing)
                }

                // Section 3: Endpoints Info
                Section("Active Endpoints") {
                    LabeledContent("GraphQL", value: APIConfiguration.shared.graphqlURL.absoluteString)
                        .font(.caption)
                    LabeledContent("WebSocket", value: APIConfiguration.shared.websocketURL.absoluteString)
                        .font(.caption)
                    LabeledContent("Upload", value: APIConfiguration.shared.uploadURL.absoluteString)
                        .font(.caption)
                }

                // Section 4: Session & Security
                Section("Session & Security") {
                    if let user = appState.currentUser {
                        HStack {
                            Text("Account")
                            Spacer()
                            Text("@\(user.username)")
                                .foregroundStyle(.secondary)
                        }

                        HStack {
                            Text("Two-Factor Auth (TOTP)")
                            Spacer()
                            Text(user.is2faEnabled ? "Enabled" : "Disabled")
                                .foregroundStyle(user.is2faEnabled ? .green : .secondary)
                        }
                    }

                    Button("Clear Stored Tokens") {
                        KeychainHelper.shared.clearTokens()
                    }
                    .foregroundStyle(.orange)

                    Button(role: .destructive) {
                        Task {
                            await appState.logout()
                            dismiss()
                        }
                    } label: {
                        HStack {
                            Spacer()
                            Text("Log Out")
                                .fontWeight(.semibold)
                            Spacer()
                        }
                    }
                }
            }
            .navigationTitle("Settings")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                ToolbarItem(placement: .automatic) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
        }
    }

    private func probeServerNow() {
        isProbing = true
        Task {
            let (live, ready, msg) = await FerroService.shared.checkBackendStatus()
            await MainActor.run {
                appState.isServerLive = live
                appState.isServerReady = ready
                appState.serverStatusMessage = msg
                probeResult = msg
                isProbing = false
            }
        }
    }
}
