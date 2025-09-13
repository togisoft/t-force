# 1. Build stage
FROM node:20-alpine AS builder

WORKDIR /app

# Install dependencies
COPY package.json package-lock.json* ./
RUN npm ci

# Copy source files and build
COPY . .
RUN npm run build

# 2. Production stage
FROM node:20-alpine AS runner

WORKDIR /app

# Sadece production için gerekli dosyaları kopyala
COPY --from=builder /app/package.json ./package.json
COPY --from=builder /app/package-lock.json ./package-lock.json
COPY --from=builder /app/node_modules ./node_modules
COPY --from=builder /app/.next ./.next
COPY --from=builder /app/public ./public
COPY --from=builder /app/next.config.js ./next.config.js

# Portu aç
EXPOSE 3000

# Production start komutu
CMD ["npm", "run", "start"]
