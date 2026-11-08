import SwiftUI

public struct CreatePostView: View {
    @Environment(\.dismiss) private var dismiss
    let onPostCreated: (Post) -> Void

    @State private var content = ""
    @State private var audience: PostAudienceGql = .public
    @State private var showPollEditor = false
    @State private var pollQuestion = ""
    @State private var pollOption1 = ""
    @State private var pollOption2 = ""
    @State private var pollOption3 = ""
    @State private var pollOption4 = ""
    @State private var isSubmitting = false
    @State private var errorMessage: String? = nil
    @State private var showError = false

    public init(onPostCreated: @escaping (Post) -> Void) {
        self.onPostCreated = onPostCreated
    }

    public var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextEditor(text: $content)
                        .frame(minHeight: 120)
                        .overlay(alignment: .topLeading) {
                            if content.isEmpty {
                                Text("What's happening in the spatial realm?")
                                    .foregroundStyle(.tertiary)
                                    .padding(.top, 8)
                                    .padding(.leading, 5)
                            }
                        }
                }

                Section("Audience") {
                    Picker("Audience", selection: $audience) {
                        ForEach(PostAudienceGql.allCases, id: \.self) { aud in
                            Text(aud.title).tag(aud)
                        }
                    }
                    .pickerStyle(.segmented)
                }

                Section {
                    Toggle("Attach Poll", isOn: $showPollEditor.animation())

                    if showPollEditor {
                        TextField("Poll Question", text: $pollQuestion)
                        TextField("Option 1", text: $pollOption1)
                        TextField("Option 2", text: $pollOption2)
                        TextField("Option 3 (optional)", text: $pollOption3)
                        TextField("Option 4 (optional)", text: $pollOption4)
                    }
                }
            }
            .navigationTitle("New Post")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("Cancel") {
                        dismiss()
                    }
                }

                ToolbarItem(placement: .topBarTrailing) {
                    Button {
                        submitPost()
                    } label: {
                        if isSubmitting {
                            ProgressView()
                        } else {
                            Text("Post")
                                .fontWeight(.bold)
                        }
                    }
                    .disabled(content.trimmingCharacters(in: .whitespaces).isEmpty || isSubmitting)
                }
            }
            .alert("Failed to Publish Post", isPresented: $showError) {
                Button("OK", role: .cancel) {}
            } message: {
                Text(errorMessage ?? "An unexpected error occurred.")
            }
        }
    }

    private func submitPost() {
        let trimmed = content.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }

        isSubmitting = true
        Task {
            do {
                let options = showPollEditor ? [pollOption1, pollOption2, pollOption3, pollOption4].filter { !$0.trimmingCharacters(in: .whitespaces).isEmpty } : []
                let newPost = try await FerroService.shared.createPost(
                    content: trimmed,
                    audience: audience,
                    pollQuestion: showPollEditor && !pollQuestion.isEmpty ? pollQuestion : nil,
                    pollOptions: options
                )
                await MainActor.run {
                    onPostCreated(newPost)
                    isSubmitting = false
                    dismiss()
                }
            } catch {
                await MainActor.run {
                    self.errorMessage = error.localizedDescription
                    self.showError = true
                    self.isSubmitting = false
                }
            }
        }
    }
}
