# 🔐 AuthForce - Modern Authentication & Chat Platform

<div align="center">
  <img src="./assets/logo.svg" alt="AuthForce Logo" width="200" height="200">
  
  [![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
  [![Next.js](https://img.shields.io/badge/Next.js-15.5.0-black.svg)](https://nextjs.org/)
  [![TypeScript](https://img.shields.io/badge/TypeScript-5.0+-blue.svg)](https://www.typescriptlang.org/)
  [![PostgreSQL](https://img.shields.io/badge/PostgreSQL-16-blue.svg)](https://www.postgresql.org/)
  [![Docker](https://img.shields.io/badge/Docker-Compose-blue.svg)](https://www.docker.com/)
  [![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
</div>

## 📸 Screenshots

<div align="center">
  <img src="./assets/screenshot.jpg" alt="AuthForce Dashboard" width="100%" style="max-width: 1000px; border-radius: 8px; box-shadow: 0 4px 20px rgba(0,0,0,0.1);">
</div>
<div align="center">
  <img src="./assets/screenshot.png" alt="AuthForce Dashboard" width="100%" style="max-width: 1000px; border-radius: 8px; box-shadow: 0 4px 20px rgba(0,0,0,0.1);">
</div>
## 📋 Table of Contents

- [🎯 About the Project](#-about-the-project)
- [✨ Features](#-features)
- [🛠️ Technology Stack](#️-technology-stack)
- [🏗️ Architecture](#️-architecture)
- [🚀 Installation](#-installation)
- [📱 Usage](#-usage)
- [🔧 Development](#-development)
- [📊 API Documentation](#-api-documentation)
- [🔒 Security](#-security)
- [🤝 Contributing](#-contributing)
- [📄 License](#-license)

## 🎯 About the Project

AuthForce is a comprehensive authentication and real-time chat platform developed for modern web applications. It provides a high-performance, secure, and scalable solution using Rust backend and Next.js frontend.

### 🎯 Goals
- **Security**: Modern encryption and authentication standards
- **Performance**: Rust's speed and Next.js optimizations
- **Scalability**: Microservice architecture and Docker support
- **User Experience**: Modern UI/UX design

### 📊 Performance Metrics
- **API Response Time**: < 50ms (average)
- **WebSocket Latency**: < 10ms
- **Memory Usage**: ~50MB (backend), ~100MB (frontend)
- **Concurrent Users**: 1000+ (single instance)
- **Database Connections**: Pool-based connection management
- **File Upload**: Chunked upload support (max 100MB)

## ✨ Features

### 🔐 Authentication
- ✅ **Email/Password Login**: Secure user registration and login
- ✅ **OAuth Integration**: Social login with Google and GitHub
- ✅ **Two-Factor Authentication (2FA)**: TOTP-based security
- ✅ **Password Reset**: Secure password reset via email
- ✅ **Session Management**: Multi-session control and termination
- ✅ **JWT Token System**: Secure token-based authentication

### 💬 Real-Time Chat
- ✅ **WebSocket Connection**: Instant messaging
- ✅ **Room System**: Private and public chat rooms
- ✅ **Media Sharing**: Image, video, and voice messages
- ✅ **Message History**: Persistent message storage
- ✅ **Room Passwords**: Secure private rooms
- ✅ **Auto Reconnection**: Automatic recovery on connection loss

### 👥 User Management
- ✅ **Profile Management**: User information and profile pictures
- ✅ **Role-Based Authorization**: Admin and user roles
- ✅ **User Status**: Active/inactive user control
- ✅ **Admin Panel**: User management and system control

### 🛡️ Security Features
- ✅ **Rate Limiting**: API request rate limiting
- ✅ **CORS Protection**: Cross-origin request security
- ✅ **SQL Injection Protection**: ORM-based secure database operations
- ✅ **XSS Protection**: Frontend security measures
- ✅ **Password Hashing**: Secure password storage with Argon2

## 🛠️ Technology Stack

### 🦀 Backend (Rust)
```toml
# Core Framework
actix-web = "4.4.0"          # Web framework
actix-cors = "0.7.1"         # CORS middleware
actix-web-actors = "4.2.0"   # WebSocket support

# Database
sea-orm = "1.1.14"           # ORM
postgres = "16-alpine"       # Database

# Authentication
jsonwebtoken = "9.3.1"       # JWT token
argon2 = "0.5.2"             # Password hashing
oauth2 = "4.4.2"             # OAuth integration

# Other
serde = "1.0.188"            # Serialization
tokio = "1.32.0"             # Async runtime
lettre = "0.11.18"           # Email sending
```

### ⚛️ Frontend (Next.js)
```json
{
  "next": "15.5.0",              // React framework
  "react": "19.0.0",            // UI library
  "typescript": "5.0+",         // Type safety
  "tailwindcss": "3.4.0",       // CSS framework
  "shadcn/ui": "latest",        // UI components
  "react-hook-form": "7.62.0",  // Form management
  "jwt-decode": "4.0.0",        // JWT parsing
  "lucide-react": "0.541.0"     // Icons
}
```

### 🗄️ Database & Infrastructure
- **PostgreSQL 16**: Primary database
- **Docker & Docker Compose**: Containerization
- **Traefik**: Reverse proxy and load balancer
- **Redis** (optional): Cache and session store
- **Nginx** (production): Static file serving
- **Let's Encrypt**: Automatic SSL certificates

### 🔄 DevOps & Monitoring
- **GitHub Actions**: CI/CD pipeline
- **Prometheus**: Metrics collection
- **Grafana**: Monitoring dashboard
- **Docker Health Checks**: Service monitoring
- **Automated Backups**: Database backup scripts
- **Log Aggregation**: Structured logging

### 🛠️ Development Tools
- **Cargo**: Rust package manager
- **npm/yarn**: Node.js package manager
- **ESLint & Prettier**: Code formatting
- **TypeScript**: Type checking
- **Sea-ORM**: Database ORM
- **Actix-Web**: Web framework
- **Tailwind CSS**: Utility-first CSS
- **Shadcn/UI**: Component library

## 🏗️ Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │    Traefik      │    │    Backend      │
│   (Next.js)     │◄──►│  (Reverse Proxy)│◄──►│    (Rust)       │
│   Port: 3000    │    │   Port: 80      │    │   Port: 8080    │
└─────────────────┘    └─────────────────┘    └─────────────────┘
         │                       │                       │
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Static Files  │    │   Load Balancer │    │   PostgreSQL    │
│   & Assets      │    │   & SSL/TLS     │    │   Database      │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### 📡 API Endpoints

#### 🔐 Authentication
```
POST   /api/auth/register     # Kullanıcı kaydı
POST   /api/auth/login        # Giriş yapma
POST   /api/auth/logout       # Çıkış yapma
GET    /api/auth/validate     # Token doğrulama
POST   /api/auth/forgot       # Şifre sıfırlama isteği
POST   /api/auth/reset        # Şifre sıfırlama
```

#### 👤 User Management
```
GET    /api/user/me           # Kullanıcı bilgileri
PUT    /api/user/profile      # Profil güncelleme
POST   /api/user/avatar       # Profil fotoğrafı
GET    /api/user/sessions     # Aktif oturumlar
DELETE /api/user/session/:id  # Oturum sonlandırma
```

#### 💬 Chat System
```
GET    /api/chat/rooms        # Sohbet odaları
POST   /api/chat/rooms        # Oda oluşturma
GET    /api/chat/rooms/:id    # Oda detayları
DELETE /api/chat/rooms/:id    # Oda silme
GET    /api/chat/messages     # Mesaj geçmişi
POST   /api/chat/upload       # Medya yükleme
WS     /ws                    # WebSocket bağlantısı
```

## 🚀 Installation

### 📋 Requirements
- **Docker & Docker Compose**: Container management
- **Git**: Version control
- **Node.js 18+**: Frontend development (optional)
- **Rust 1.70+**: Backend development (optional)

### 🔧 Quick Start

1. **Clone the project**
```bash
git clone https://github.com/yourusername/authforce.git
cd authforce
```

2. **Create environment file**
```bash
cp .env.template .env
# Edit the .env file
```

3. **Start development environment**
```bash
./scripts/dev.sh
```

4. **Access the application**
- Frontend: http://localhost
- Backend API: http://localhost/api
- Traefik Dashboard: http://localhost:8080

### 🐳 Production Deployment

```bash
# Create production environment file
cp env.production.template .env.production

# Start in production mode
./scripts/prod.sh
```

## 📱 Usage

### 👤 User Registration and Login
1. Click on "Sign Up" tab on the main page
2. Enter email, name and a strong password
3. Complete email verification
4. Sign in or use OAuth for quick login

### 💬 Chat Usage
1. Access the dashboard
2. Click on "New Room" button
3. Set room name and password (optional)
4. Invite friends by sharing the room code
5. Start sending messages, images, videos and audio

### 🔒 Security Settings
1. Go to profile settings
2. Open "Two-Factor Authentication" section
3. Scan QR code and enter verification code
4. Store your backup codes in a safe place

## 🔧 Development

### 🔧 Development Environment Setup

```bash
# Backend development
cd backend
cargo run

# Frontend development
cd frontend
npm install
npm run dev

# Database migration
./scripts/migrate.sh
```

### 🧪 Running Tests

```bash
# Backend tests
cd backend
cargo test

# Frontend tests
cd frontend
npm test

# E2E tests
npm run test:e2e
```

### 📜 Available Scripts

All scripts should be run from the project root directory:

```bash
# Start development environment
./scripts/dev.sh

# Start production environment
./scripts/prod.sh

# Run database migrations
./scripts/migrate.sh

# Run test suite
./scripts/test.sh

# Create backup
./scripts/backup.sh

# Check system health
./scripts/health-check.sh

# Chat system testing
./scripts/test_chat.sh
```

### 📝 Code Standards

#### Rust (Backend)
```bash
# Format check
cargo fmt --check

# Linting
cargo clippy -- -D warnings

# Run tests
cargo test
```

#### TypeScript (Frontend)
```bash
# Format check
npm run lint

# Type checking
npm run type-check

# Build test
npm run build
```



## 📊 API Documentation

### 🔐 Authentication Endpoints

#### POST /api/auth/register
```json
{
  "name": "John Doe",
  "email": "john@example.com",
  "password": "securePassword123"
}
```

#### POST /api/auth/login
```json
{
  "email": "john@example.com",
  "password": "securePassword123"
}
```

**Response:**
```json
{
  "user": {
    "id": "uuid",
    "email": "john@example.com",
    "name": "John Doe",
    "role": "user"
  },
  "token": "jwt_token_here"
}
```

### 💬 Chat WebSocket Events

#### Client → Server
```json
{
  "type": "join_room",
  "room_id": "room_uuid",
  "password": "optional_password"
}

{
  "type": "send_message",
  "room_id": "room_uuid",
  "content": "Hello, world!",
  "message_type": "text"
}
```

#### Server → Client
```json
{
  "type": "message",
  "data": {
    "id": "message_uuid",
    "user": {
      "id": "user_uuid",
      "name": "John Doe"
    },
    "content": "Hello, world!",
    "timestamp": "2024-01-01T12:00:00Z"
  }
}
```

## 🔒 Security

### 🛡️ Security Measures
- **Password Hashing**: Argon2 algorithm
- **JWT Token**: Secure token-based authentication
- **Rate Limiting**: API request rate limiting
- **CORS**: Cross-origin request security
- **SQL Injection**: Protection via ORM usage
- **XSS**: Frontend sanitization

### 🔐 Environment Variables
```bash
# Critical security settings
NEXTAUTH_SECRET=your-super-secret-key
JWT_SECRET=your-jwt-secret
DATABASE_URL=postgres://user:pass@localhost/db

# OAuth settings
GOOGLE_CLIENT_ID=your-google-client-id
GOOGLE_CLIENT_SECRET=your-google-client-secret
GITHUB_CLIENT_ID=your-github-client-id
GITHUB_CLIENT_SECRET=your-github-client-secret
```

### 🚨 Security Recommendations
1. Use **strong passwords**
2. **Enable 2FA**
3. Keep **environment variables** secure
4. Use **HTTPS** (in production)
5. Perform **regular updates**

## 🤝 Contributing

1. **Fork** the project
2. Create a **feature branch** (`git checkout -b feature/amazing-feature`)
3. **Commit** your changes (`git commit -m 'Add amazing feature'`)
4. **Push** to the branch (`git push origin feature/amazing-feature`)
5. Open a **Pull Request**

### 📋 Contribution Guidelines
- Follow code standards
- Write tests
- Update documentation
- Write descriptive commit messages

## 📄 License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

---

<div align="center">
  <p>Developed with ❤️</p>
  <p>
    <a href="https://github.com/yourusername/authforce/issues">🐛 Bug Report</a> |
    <a href="https://github.com/yourusername/authforce/discussions">💬 Discussions</a> |
    <a href="https://github.com/yourusername/authforce/wiki">📖 Wiki</a>
  </p>
</div>