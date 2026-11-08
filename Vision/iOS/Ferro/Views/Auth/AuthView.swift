import SwiftUI

public struct AuthView: View {
    @Environment(AppState.self) private var appState

    @State private var isSignUp = false
    @State private var usernameOrEmail = ""
    @State private var password = ""
    @State private var username = ""
    @State private var email = ""
    @State private var displayName = ""
    @State private var isPasswordVisible = false
    @State private var isLoading = false

    public init() {}

    public var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 28) {
                    // App Logo & Header
                    VStack(spacing: 12) {
                        Image(systemName: "circle.hexagongrid.fill")
                            .font(.system(size: 64))
                            .foregroundStyle(.linearGradient(
                                colors: [.indigo, .purple, .cyan],
                                startPoint: .topLeading,
                                endPoint: .bottomTrailing
                            ))
                            .padding(.top, 24)

                        Text("Ferro")
                            .font(.system(size: 38, weight: .bold, design: .rounded))

                        Text("High-Performance Spatial Social Network")
                            .font(.subheadline)
                            .foregroundStyle(.secondary)
                    }

                    // Mode Picker (Login vs Sign Up)
                    Picker("Auth Mode", selection: $isSignUp) {
                        Text("Log In").tag(false)
                        Text("Sign Up").tag(true)
                    }
                    .pickerStyle(.segmented)
                    .padding(.horizontal)

                    // Input Fields
                    VStack(spacing: 16) {
                        if !isSignUp {
                            // Login Fields
                            HStack {
                                Image(systemName: "person.crop.circle")
                                    .foregroundStyle(.secondary)
                                #if os(iOS)
                                TextField("Username or Email", text: $usernameOrEmail)
                                    .textInputAutocapitalization(.never)
                                    .autocorrectionDisabled()
                                #else
                                TextField("Username or Email", text: $usernameOrEmail)
                                    .autocorrectionDisabled()
                                #endif
                            }
                            .padding()
                            .background(Color(uiColor: .secondarySystemBackground))
                            .clipShape(RoundedRectangle(cornerRadius: 12))

                            HStack {
                                Image(systemName: "lock")
                                    .foregroundStyle(.secondary)
                                if isPasswordVisible {
                                    TextField("Password", text: $password)
                                } else {
                                    SecureField("Password", text: $password)
                                }
                                Button {
                                    isPasswordVisible.toggle()
                                } label: {
                                    Image(systemName: isPasswordVisible ? "eye.slash" : "eye")
                                        .foregroundStyle(.secondary)
                                }
                            }
                            .padding()
                            .background(Color(uiColor: .secondarySystemBackground))
                            .clipShape(RoundedRectangle(cornerRadius: 12))
                        } else {
                            // Sign Up Fields
                            HStack {
                                Image(systemName: "person")
                                    .foregroundStyle(.secondary)
                                TextField("Display Name", text: $displayName)
                            }
                            .padding()
                            .background(Color(uiColor: .secondarySystemBackground))
                            .clipShape(RoundedRectangle(cornerRadius: 12))

                            HStack {
                                Image(systemName: "at")
                                    .foregroundStyle(.secondary)
                                #if os(iOS)
                                TextField("Username", text: $username)
                                    .textInputAutocapitalization(.never)
                                    .autocorrectionDisabled()
                                #else
                                TextField("Username", text: $username)
                                    .autocorrectionDisabled()
                                #endif
                            }
                            .padding()
                            .background(Color(uiColor: .secondarySystemBackground))
                            .clipShape(RoundedRectangle(cornerRadius: 12))

                            HStack {
                                Image(systemName: "envelope")
                                    .foregroundStyle(.secondary)
                                #if os(iOS)
                                TextField("Email", text: $email)
                                    .textInputAutocapitalization(.never)
                                    .autocorrectionDisabled()
                                    .keyboardType(.emailAddress)
                                #else
                                TextField("Email", text: $email)
                                    .autocorrectionDisabled()
                                #endif
                            }
                            .padding()
                            .background(Color(uiColor: .secondarySystemBackground))
                            .clipShape(RoundedRectangle(cornerRadius: 12))

                            HStack {
                                Image(systemName: "lock")
                                    .foregroundStyle(.secondary)
                                if isPasswordVisible {
                                    TextField("Password (min 8 chars)", text: $password)
                                } else {
                                    SecureField("Password (min 8 chars)", text: $password)
                                }
                                Button {
                                    isPasswordVisible.toggle()
                                } label: {
                                    Image(systemName: isPasswordVisible ? "eye.slash" : "eye")
                                        .foregroundStyle(.secondary)
                                }
                            }
                            .padding()
                            .background(Color(uiColor: .secondarySystemBackground))
                            .clipShape(RoundedRectangle(cornerRadius: 12))
                        }

                        // Submit Button
                        Button {
                            handleAuthAction()
                        } label: {
                            HStack {
                                if isLoading {
                                    ProgressView()
                                        .tint(.white)
                                } else {
                                    Text(isSignUp ? "Create Account" : "Log In")
                                        .fontWeight(.semibold)
                                }
                            }
                            .frame(maxWidth: .infinity)
                            .padding()
                            .background(.linearGradient(
                                colors: [.indigo, .purple],
                                startPoint: .leading,
                                endPoint: .trailing
                            ))
                            .foregroundStyle(.white)
                            .clipShape(RoundedRectangle(cornerRadius: 12))
                        }
                        .disabled(isLoading || (isSignUp ? (username.isEmpty || email.isEmpty || password.isEmpty) : (usernameOrEmail.isEmpty || password.isEmpty)))
                    }
                    .padding(.horizontal)

                    // Quick Demo Accounts
                    VStack(alignment: .leading, spacing: 12) {
                        Text("QUICK DEMO ACCOUNTS")
                            .font(.caption)
                            .fontWeight(.bold)
                            .foregroundStyle(.secondary)

                        HStack(spacing: 12) {
                            quickLoginButton(name: "Lily", username: "kernelily")
                            quickLoginButton(name: "Alice", username: "alice_design")
                            quickLoginButton(name: "Bob", username: "bob_engineer")
                        }
                    }
                    .padding(.horizontal)

                    // Server Probe Status Footer
                    VStack(spacing: 6) {
                        HStack(spacing: 6) {
                            Circle()
                                .fill(appState.isServerLive ? Color.green : Color.orange)
                                .frame(width: 8, height: 8)

                            Text(appState.isServerLive ? "Live Serve API Backend" : "Connecting to Serve API...")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }

                        Text(appState.serverStatusMessage)
                            .font(.caption2)
                            .foregroundStyle(.tertiary)
                            .multilineTextAlignment(.center)
                            .padding(.horizontal)
                    }
                    .padding(.top, 16)
                }
                .padding(.bottom, 32)
            }
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
        }
    }

    private func quickLoginButton(name: String, username: String) -> some View {
        Button {
            self.usernameOrEmail = username
            self.password = "password123"
            handleAuthAction()
        } label: {
            VStack(spacing: 4) {
                Image(systemName: "person.crop.circle.badge.checkmark")
                    .font(.title3)
                Text(name)
                    .font(.footnote)
                    .fontWeight(.medium)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 10)
            .background(Color(uiColor: .tertiarySystemBackground))
            .clipShape(RoundedRectangle(cornerRadius: 10))
        }
    }

    private func handleAuthAction() {
        isLoading = true
        Task {
            if isSignUp {
                await appState.signup(
                    username: username,
                    email: email,
                    password: password,
                    displayName: displayName
                )
            } else {
                await appState.login(
                    usernameOrEmail: usernameOrEmail,
                    password: password
                )
            }
            isLoading = false
        }
    }
}
