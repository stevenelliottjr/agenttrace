# AgentTrace iOS Companion App

A native iOS companion app for AgentTrace that provides real-time AI agent observability on mobile devices.

## Requirements

- iOS 17.0+
- Xcode 15.0+
- XcodeGen (for project generation)

## Setup

### 1. Install XcodeGen

```bash
brew install xcodegen
```

### 2. Generate Xcode Project

```bash
cd ios/AgentTrace
xcodegen generate
```

### 3. Open in Xcode

```bash
open AgentTrace.xcodeproj
```

### 4. Configure Signing

1. Open the project in Xcode
2. Select the AgentTrace target
3. Go to "Signing & Capabilities"
4. Select your development team
5. Repeat for AgentTraceWidgets and AgentTraceLiveActivity targets

### 5. Build and Run

Select your target device or simulator and press ⌘R to build and run.

## Project Structure

```
ios/AgentTrace/
├── AgentTrace/                 # Main app target
│   ├── App/                    # App entry point
│   ├── Core/                   # Extensions and utilities
│   ├── Features/               # Feature modules
│   │   ├── Dashboard/          # Dashboard view and view model
│   │   ├── Traces/             # Trace list and detail views
│   │   ├── Metrics/            # Metrics charts
│   │   ├── Alerts/             # Alerts management
│   │   └── Settings/           # App settings
│   ├── Models/                 # Data models
│   │   ├── Domain/             # Core domain types
│   │   ├── DTOs/               # API response types
│   │   └── SwiftData/          # Persistence models
│   ├── Services/               # Business logic services
│   │   ├── Networking/         # API and SSE clients
│   │   ├── Persistence/        # SwiftData manager
│   │   └── Notifications/      # Push notification handling
│   └── Navigation/             # Navigation coordination
├── AgentTraceWidgets/          # Widget extension
├── AgentTraceLiveActivity/     # Live Activity extension
├── Shared/                     # Code shared between targets
└── project.yml                 # XcodeGen configuration
```

## Features

### Dashboard
- Real-time metrics summary (cost, tokens, latency, errors)
- Recent traces list with status indicators
- Active alerts section
- Live connection status indicator

### Traces
- Searchable/filterable trace list
- Trace detail with waterfall visualization
- Span detail bottom sheet
- Pull-to-refresh and infinite scroll

### Metrics (Swift Charts)
- Cost breakdown pie chart by model
- Latency over time with P50/P95/P99 bands
- Token usage stacked bar chart
- Error rate trends

### Alerts
- Alert rules list
- Triggered alerts with acknowledge action
- Filter by severity/status

### iOS-Specific Features
- **Live Activities**: Running agent session on lock screen
- **Widgets**: Quick metrics glance (small/medium/large)
- **Offline Support**: SwiftData caching
- **Deep Links**: `agenttrace://trace/{id}`, `agenttrace://alert/{id}`

## Configuration

### Server URL

The app connects to `http://localhost:8080` by default. To change:

1. Open the app
2. Go to Settings tab
3. Enter your server URL
4. Tap "Test Connection" to verify

### Deep Links

The app supports the following URL schemes:

| URL | Action |
|-----|--------|
| `agenttrace://dashboard` | Open Dashboard |
| `agenttrace://traces` | Open Traces list |
| `agenttrace://trace/{id}` | Open specific trace |
| `agenttrace://metrics` | Open Metrics |
| `agenttrace://alerts` | Open Alerts |
| `agenttrace://alert/{id}` | Open specific alert |
| `agenttrace://settings` | Open Settings |

## Architecture

### State Management
- Uses iOS 17 `@Observable` macro
- MVVM pattern with ViewModels per feature
- Centralized navigation coordination

### Networking
- `APIClient` actor for async/await REST calls
- `SSEClient` for real-time span streaming
- Automatic reconnection on disconnect

### Persistence
- SwiftData for offline caching
- App Groups for widget/Live Activity data sharing
- 24-hour cache expiration

## API Endpoints

The app uses the following backend endpoints:

| Feature | Endpoint |
|---------|----------|
| Dashboard | `GET /api/v1/metrics/summary`, `GET /api/v1/traces?limit=20` |
| Traces | `GET /api/v1/traces`, `GET /api/v1/traces/:id` |
| Spans | `GET /api/v1/spans`, `GET /api/v1/spans/:id` |
| Metrics | `GET /api/v1/metrics/costs`, `/latency`, `/errors` |
| Alerts | `GET /api/v1/alerts/rules`, `GET /api/v1/alerts/events` |
| Real-time | `GET /api/v1/stream` (SSE) |
| Acknowledge | `POST /api/v1/alerts/events/:id/acknowledge` |

## Development

### Adding a New Feature

1. Create a new folder under `Features/`
2. Add ViewModel with `@Observable` macro
3. Create SwiftUI views
4. Add navigation in `ContentView.swift`

### Testing the SSE Stream

1. Start the AgentTrace collector locally
2. Run the app in simulator
3. Start an agent in another terminal
4. Spans should appear in real-time on the Dashboard

## License

MIT License - see LICENSE file for details.
