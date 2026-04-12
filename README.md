# LogiTrack 🚚📦

LogiTrack is a **Tracking & Notification Service** designed to monitor shipment deliveries from expedition/logistics services and notify users in real time when shipment statuses change.

This project aims to provide a **centralized and extensible tracking system**, independent of any single expedition provider, making it suitable for integration with multiple logistics services.

> 🚀 Status: Active Development & Implemented  
> LogiTrack is now a fully functional microservices-based platform built with Rust.

---

## ✨ Features

- 📍 **Centralized Shipment Tracking**: Track shipments using air waybill (AWB) numbers across various couriers via Biteship.
- 🔔 **Real-time Notifications**: Automated alerts sent via Email (SMTP) and WhatsApp (WAHA) regarding shipment status changes.
- 🔄 **Async Polling & Webhooks**: Supports both scheduled polling for shipment updates and real-time inbound webhook processing from logistics providers.
- 💳 **Wallet & Billing Interface**: Integrated Top-up and virtual wallet management with automated ledger deductions for tracking activities.
- 🔗 **Third-Party Integrations**: Fully integrated with Biteship (Logistics) and Xendit (Payment Gateway).
- 📊 **Monitoring & Observability**: Complete metric tracking with Prometheus and visualization via Grafana dashboard.
- 🔐 **Robust Security**: Uses JWT for user sessions and API Keys with Redis caching for machine-to-machine integrations.

---

## 🏗️ Microservices Architecture

LogiTrack relies on a highly scalable, event-driven microservices architecture:

1. **`gateway-service`**: The main entry point acting as a reverse proxy to route traffic to the appropriate domain service.
2. **`auth-service`**: Handles JWT generation, User management, and robust API Key validation.
3. **`tracking-service`**: Manages shipment CRUD, subscriptions, and chronological tracking events.
4. **`webhook-service`**: Securely receives and processes inbound webhooks from Biteship and Xendit.
5. **`topup-service`**: Handles wallet balances, payment transactions, and pricing tiers.
6. **`billing-service`**: Acts as a background consumer for billing-related message queues (RabbitMQ).
7. **`notification-service`**: Listens to messaging queues and dispatches Email or WhatsApp alerts based on user preference.
8. **`polling-service`**: A cron-like worker that periodically queries Biteship for external shipment updates.

---

## 🛠️ Tech Stack

- **Core Backend**: Rust 🦀, Axum (Web Framework), Tokio (Async Runtime)
- **Database**: PostgreSQL (with SQLx for compile-time query checks)
- **Message Broker**: RabbitMQ (Event-driven asynchronous jobs)
- **Caching**: Redis
- **Observability**: Prometheus & Grafana (Service health, latency, queue metrics)
- **Containerization**: Docker & Docker Compose

---

## 🚀 Getting Started

The entire suite of services, databases, and message brokers runs seamlessly inside Docker.

### Prerequisites
- Docker & Docker Compose
- Environment file `.env` populated with required API keys (`BITESHIP`, `XENDIT`, `SMTP`, `WAHA`)

### Running the application
```bash
docker-compose up --build -d
```
Once running, the Gateway will be accessible at `http://localhost:3000`. 
- Grafana: `http://localhost:3100` (for monitoring)
- RabbitMQ Management: `http://localhost:15672`

---

## 📖 API Documentation

The complete OpenAPI specification is available in the [`openapi.yaml`](./openapi.yaml) file, outlining all public and protected routes across the services.

---

## 📄 License

This project is licensed under the Apache 2.0 License.
