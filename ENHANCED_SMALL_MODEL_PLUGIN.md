# Enhanced Small Model Configuration Plugin - Project Plan

## Overview

This project creates an OpenCode plugin that provides comprehensive small_model configuration capabilities through web-based interface and custom tools, addressing the current limitation where small_model cannot be set through the Desktop GUI.

## Project Goals

- [ ] Enable persistent small_model configuration in opencode.jsonc
- [ ] Provide intuitive web-based settings interface
- [ ] Offer model discovery and recommendation system
- [ ] Support model presets for different workflows
- [ ] Create seamless user experience with multiple access methods

## Technical Architecture

### Core Components

```
.more-preferences/
├── .opencode/
│   ├── plugin/
│   │   ├── small-model-config.ts      # Main plugin entry point
│   │   ├── model-discovery.ts        # Available model detection
│   │   ├── config-manager.ts        # opencode.jsonc manipulation
│   │   ├── web-server.ts          # Web UI server
│   │   ├── preset-manager.ts       # Model preset management
│   │   └── recommendation-engine.ts  # Smart model suggestions
│   └── command/
│       └── small-model.md           # Slash command definition
├── web-ui/                         # Settings interface
│   ├── index.html                  # Main settings page
│   ├── components/
│   │   ├── model-selector.js       # Model selection component
│   │   ├── preset-manager.js       # Preset management
│   │   ├── model-info.js          # Model details display
│   │   └── config-validator.js    # Input validation
│   ├── style.css                  # UI styling
│   └── assets/
│       └── icons/                # UI icons
├── tests/                         # Test suite
│   ├── config-manager.test.ts
│   ├── model-discovery.test.ts
│   └── web-server.test.ts
├── docs/                         # Documentation
│   ├── API.md                     # API documentation
│   ├── INSTALLATION.md             # Setup instructions
│   └── USER_GUIDE.md             # Usage guide
├── package.json                   # Dependencies
├── tsconfig.json                 # TypeScript configuration
└── README.md                     # Project overview
```

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1)

#### 1.1 Plugin Foundation
- [ ] Set up plugin structure following OpenCode patterns
- [ ] Create TypeScript configuration and build pipeline
- [ ] Implement basic plugin registration and context handling
- [ ] Set up error handling and logging framework

#### 1.2 Configuration Management
- [ ] Implement `config-manager.ts` for opencode.jsonc manipulation
- [ ] Add safe file I/O with backup/restore functionality
- [ ] Create configuration validation and schema checking
- [ ] Implement atomic updates to prevent corruption

#### 1.3 Command Interface
- [ ] Create `.opencode/command/small-model.md` definition
- [ ] Implement basic slash commands: set, get, list
- [ ] Add command parsing and validation
- [ ] Create help system and usage documentation

**Deliverables:**
- Working plugin with basic small_model set/get functionality
- Safe configuration file manipulation
- Command-line interface for basic operations

### Phase 2: Model Discovery & Selection (Week 2)

#### 2.1 Model Discovery System
- [ ] Implement `model-discovery.ts` for provider enumeration
- [ ] Add model capability detection (speed, cost, context window)
- [ ] Create model categorization (fast, balanced, capable)
- [ ] Implement model availability validation

#### 2.2 Web Server Foundation
- [ ] Set up Express server in `web-server.ts`
- [ ] Create REST API endpoints for configuration
- [ ] Implement CORS and security middleware
- [ ] Add server status and health checks

#### 2.3 Basic Web UI
- [ ] Create responsive HTML layout in `index.html`
- [ ] Implement model selection dropdown with search
- [ ] Add configuration save/load functionality
- [ ] Create mobile-friendly responsive design

**Deliverables:**
- Full model discovery system
- Working web interface with model selection
- REST API for configuration management
- Mobile-responsive UI

### Phase 3: Advanced Features (Week 3)

#### 3.1 Preset Management
- [ ] Implement `preset-manager.ts` for save/load functionality
- [ ] Create preset schema and validation
- [ ] Add built-in presets (fast, balanced, capable)
- [ ] Implement preset import/export functionality

#### 3.2 Recommendation Engine
- [ ] Create `recommendation-engine.ts` for smart suggestions
- [ ] Analyze user's primary model selection
- [ ] Implement cost/performance trade-off analysis
- [ ] Add learning from user preferences

#### 3.3 Enhanced Web UI
- [ ] Add preset management interface
- [ ] Implement model comparison tool
- [ ] Create usage analytics dashboard
- [ ] Add real-time validation feedback

**Deliverables:**
- Complete preset management system
- Intelligent model recommendations
- Advanced web UI with analytics
- Model comparison tools

### Phase 4: Polish & Integration (Week 4)

#### 4.1 Testing & Validation
- [ ] Write comprehensive test suite
- [ ] Add integration tests with various model providers
- [ ] Perform cross-platform compatibility testing
- [ ] Conduct security and performance testing

#### 4.2 Documentation & Deployment
- [ ] Write complete API documentation
- [ ] Create installation and setup guides
- [ ] Add user tutorial and FAQ
- [ ] Prepare for npm/registry distribution

#### 4.3 User Experience Polish
- [ ] Add keyboard shortcuts and accessibility features
- [ ] Implement browser notifications for configuration changes
- [ ] Create quick-access toolbar/pop-up interface
- [ ] Add session persistence across restarts

**Deliverables:**
- Production-ready plugin with full test coverage
- Comprehensive documentation
- Polished user experience with all features
- Ready for distribution

## Core Features Specification

### 1. Command-Line Interface

```bash
/small-model set <model>           # Set small_model directly
/small-model get                   # Show current setting
/small-model list                  # List available models
/small-model gui                   # Open web settings interface
/small-model save <name>           # Save current as preset
/small-model load <name>           # Load saved preset
/small-model presets                 # List all presets
/small-model recommend              # Get model recommendations
```

### 2. Web Interface Features

#### Model Selection
- Dropdown with search and filtering
- Model information display (speed, cost, capabilities)
- Provider grouping and categorization
- Real-time availability checking

#### Configuration Management
- Visual small_model setting with validation
- Live preview of configuration changes
- Rollback functionality for failed updates
- Configuration import/export

#### Preset System
- Create custom presets for different workflows
- Built-in presets: "Development", "Testing", "Production"
- Preset sharing via import/export
- Quick preset switching from web UI

#### Analytics & Insights
- Model usage statistics and trends
- Cost analysis and budget tracking
- Performance metrics and recommendations
- Visual charts and reports

### 3. Integration Points

#### OpenCode SDK Integration
- Use `@opencode-ai/plugin` for tool registration
- Leverage client SDK for model discovery
- Hook into configuration change events
- Provide seamless session integration

#### Configuration File Management
- Safe manipulation of opencode.jsonc
- Atomic updates with rollback capability
- Configuration validation and error handling
- Backup and restore functionality

#### Provider Integration
- Support for all major OpenCode providers
- Real-time model availability checking
- Provider-specific optimization suggestions
- Automatic model discovery and caching

## Technical Specifications

### Dependencies

```json
{
  "dependencies": {
    "@opencode-ai/plugin": "latest",
    "express": "^4.18.2",
    "cors": "^2.8.5",
    "zod": "^3.22.4",
    "js-tiktoken": "^1.0.7"
  },
  "devDependencies": {
    "typescript": "^5.0.0",
    "@types/node": "^20.0.0",
    "vitest": "^1.0.0",
    "eslint": "^8.0.0"
  }
}
```

### API Endpoints

```
GET  /api/models              # List available models
GET  /api/models/:provider     # Models for specific provider
GET  /api/config              # Current configuration
POST /api/config              # Update configuration
GET  /api/presets             # List saved presets
POST /api/presets             # Create new preset
PUT  /api/presets/:id         # Update preset
DELETE /api/presets/:id        # Delete preset
GET  /api/recommendations      # Get model recommendations
```

### Configuration Schema

```typescript
interface SmallModelConfig {
  small_model?: string              // Currently selected model
  presets?: ModelPreset[]          // Saved model presets
  recommendations?: RecommendationConfig
  analytics?: AnalyticsConfig
}

interface ModelPreset {
  id: string
  name: string
  description: string
  model: string
  created: string
  lastUsed?: string
}
```

## Success Metrics

### Technical Metrics
- [ ] Plugin loads without errors in all environments
- [ ] Configuration operations complete in < 500ms
- [ ] Web UI loads in < 2 seconds
- [ ] 100% test coverage for core functionality

### User Experience Metrics
- [ ] Intuitive command interface with clear help text
- [ ] Responsive web UI that works on mobile and desktop
- [ ] Zero-configuration setup for new users
- [ ] Clear error messages and recovery options

### Integration Metrics
- [ ] Works with all OpenCode Desktop versions
- [ ] Supports all major model providers
- [ ] Maintains compatibility with existing opencode.jsonc
- [ ] Follows OpenCode plugin best practices

## Risks & Mitigations

### Technical Risks
1. **Configuration file corruption**
   - Risk: Direct file manipulation could break opencode.jsonc
   - Mitigation: Atomic updates with backup/restore functionality

2. **Provider API changes**
   - Risk: Model discovery APIs may change
   - Mitigation: Version-specific adapters and graceful fallbacks

3. **Web server conflicts**
   - Risk: Port conflicts with other plugins
   - Mitigation: Dynamic port allocation and conflict detection

### User Experience Risks
1. **Complexity overwhelm**
   - Risk: Too many features confuse users
   - Mitigation: Progressive disclosure and simple defaults

2. **Plugin compatibility**
   - Risk: May conflict with other configuration plugins
   - Mitigation: Configuration namespacing and conflict detection

## Future Enhancements

### Post-Release Features
- [ ] Multi-language support for web UI
- [ ] Integration with OpenCode Desktop settings (when API allows)
- [ ] Advanced cost tracking and budgeting
- [ ] Team/organization preset sharing
- [ ] Plugin marketplace distribution

### Platform Expansion
- [ ] Support for OpenCode Web and TUI versions
- [ ] Docker containerization for easy deployment
- [ ] CI/CD pipeline for automated updates
- [ ] Plugin telemetry and analytics

---

**Project Start Date:** January 31, 2026  
**Target Completion:** February 28, 2026  
**Primary Developer:** Plugin Development Team  
**Documentation:** Maintained in `docs/` directory  
**Issues & Tracking:** GitHub Issues (to be created)