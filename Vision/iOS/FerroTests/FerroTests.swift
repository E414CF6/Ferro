import Foundation
import Testing
@testable import Ferro

struct FerroModelTests {

    @Test func testUserDecodingAndEncoding() throws {
        let user = MockData.currentUser
        let encoder = JSONEncoder()
        let data = try encoder.encode(user)
        let decoded = try JSONDecoder().decode(User.self, from: data)

        #expect(decoded.id == user.id)
        #expect(decoded.username == "kernelily")
        #expect(decoded.is2faEnabled == true)
        #expect(decoded.isMe == true)
    }

    @Test func testPostAndPollModels() throws {
        let posts = MockData.posts
        #expect(posts.count >= 3)

        let pollPost = posts.first { $0.poll != nil }
        #expect(pollPost != nil)
        #expect(pollPost?.poll?.options.count == 3)
        #expect(pollPost?.poll?.totalVotes == 255)
    }

    @Test func testStoryLifespan() {
        let story = MockData.stories[0]
        #expect(!story.id.isEmpty)
        #expect(story.author.username == "alice_design")
        #expect(story.viewsCount > 0)
        #expect(story.isExpired == false)
    }

    @Test func testDirectMessagesAndConversations() {
        let conversations = MockData.conversations
        #expect(conversations.count >= 2)
        #expect(conversations[0].otherUser.username == "alice_design")
        #expect(conversations[0].unreadCount == 1)

        let chatMessages = MockData.chatMessagesWithAlice
        #expect(chatMessages.count == 3)
    }

    @Test func testNotificationTypes() {
        let notifs = MockData.notifications
        #expect(notifs.count >= 4)
        #expect(notifs[0].notificationType == .likePost)
        #expect(notifs[0].notificationType.actionDescription == "liked your post")
    }

    @Test func testWikiArticleGeospatial() {
        let articles = MockData.wikiArticles
        #expect(articles.count >= 2)
        #expect(articles[0].latitude == 37.7749)
        #expect(articles[0].longitude == -122.4194)
        #expect(articles[0].category == "Metaverse Architecture")
    }

    @Test func testAPIConfigurationEndpoints() {
        let config = APIConfiguration.shared
        #expect(config.baseURLString.contains("8080"))
        #expect(config.graphqlURL.absoluteString.contains("/graphql"))
        #expect(config.websocketURL.absoluteString.contains("/graphql/ws"))
        #expect(config.healthReadyURL.absoluteString.contains("/health/ready"))
        #expect(config.uploadURL.absoluteString.contains("/api/upload"))
    }
}
