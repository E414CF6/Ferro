# Vision

Vision is a responsive web client for the Ferro social networking platform. It is built with Next.js 15 (App Router),
React 19, TypeScript, and Tailwind CSS, interfacing directly with the Serve GraphQL backend service.

---

## Technical Stack

- **Framework**: Next.js 15 (App Router)
- **Library**: React 19
- **Language**: TypeScript 5
- **Styling**: Tailwind CSS
- **Icons**: Lucide React
- **API Protocol**: GraphQL (HTTP POST queries and mutations)

---

## Features

- **Ephemeral Stories (24h Auto-Expiration)**
  - Horizontal story avatar tray with unread gradient rings and read indicator states.
  - Interactive story viewer with 5-second automatic progression, pause/resume controls, and left/right tap
    navigation.
  - Story view logging (`viewStory`) and author-only viewer lists (`viewers`).
  - Real-time countdown timer display and new story creation modal.

- **Responsive Feed & Social Interactions**
  - Instant toggle between Following and Explore global feeds.
  - Inline and modal post composers supporting rich text, media attachments, and quote posts.
  - Optimistic like animations, reposts, and bookmark toggling.
  - Real-time comment submission, threaded list rendering, and post deletion controls for authors.

- **Direct Messaging (1:1 DMs)**
  - Conversation list panel featuring conversation search, latest message snippets, and unread badges.
  - Real-time 1:1 chat interface with distinct bubble alignments, timestamps, and delivery/read checkmarks.
  - Periodic polling synchronization, auto-scroll to bottom, and read status updates (`markMessagesAsRead`).
  - Direct message creation modal accessible from user profiles and recommendation cards.

- **Creator Discovery & Full-Text Search**
  - Full-text keyword search across post content (`searchPosts`).
  - Recommended creator cards with one-click follow and unfollow actions.

- **Comprehensive User Profiles**
  - Profile headers, avatars, biographies, location tags, and external website links.
  - Dynamic metrics: total posts, followers, and following count.
  - Tabbed views for created posts, liked posts, and active stories.
  - Profile edit modal for updating display name, biography, avatar, and banner images.

- **Authentication**
  - JWT-based login and registration flows with token persistence.

---

## Getting Started

### 1. Prerequisites

- Node.js 20.x or later
- npm or yarn package manager
- Running instance of the Serve API (`http://localhost:8080/graphql`)

### 2. Installation

Install project dependencies from the `Vision` directory:

```bash
npm install
```

### 3. Development Server

Start the local development server:

```bash
npm run dev
```

The application will be accessible at `http://localhost:3000`.

### 4. Production Build

To generate an optimized production build:

```bash
npm run build
npm run start
```
