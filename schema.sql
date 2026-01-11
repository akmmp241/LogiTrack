CREATE EXTENSION IF NOT EXISTS "uuid-ossp";


CREATE TYPE shipment_status AS ENUM (
    'CREATED', 'RECEIVED', 'IN_TRANSIT', 'OUT_FOR_DELIVERY',
    'DELIVERED', 'FAILED', 'RETURNED', 'CANCELLED', 'UNKNOWN'
    );

CREATE TYPE shipment_source AS ENUM ('INTERNAL', 'EXTERNAL');
CREATE TYPE tracking_event_source AS ENUM ('POLLING', 'WEBHOOK');
CREATE TYPE notification_channel AS ENUM ('WHATSAPP', 'EMAIL', 'TELEGRAM');

CREATE TABLE users
(
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name         TEXT NOT NULL,
    phone_number TEXT UNIQUE,
    email        TEXT UNIQUE,
    created_at   TIMESTAMPTZ      DEFAULT now()
);

CREATE TABLE user_notification_preferences
(
    user_id          UUID PRIMARY KEY REFERENCES users (id) ON DELETE CASCADE,

    default_channels notification_channel[] DEFAULT '{WHATSAPP, EMAIL}',

    updated_at       TIMESTAMPTZ            DEFAULT now()
);

CREATE TABLE orders
(
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id      UUID REFERENCES users (id),
    order_ref    TEXT UNIQUE NOT NULL,
    total_amount DECIMAL(12, 2),
    created_at   TIMESTAMPTZ      DEFAULT now()
);

CREATE TABLE shipments
(
    id                 UUID PRIMARY KEY         DEFAULT uuid_generate_v4(),
    waybill_id         TEXT            NOT NULL,
    courier_code       TEXT            NOT NULL,
    source             shipment_source NOT NULL,

    order_id           UUID            REFERENCES orders (id) ON DELETE SET NULL,

    external_order_ref TEXT,

    current_status     shipment_status NOT NULL DEFAULT 'CREATED',

    created_at         TIMESTAMPTZ              DEFAULT now(),
    UNIQUE (waybill_id, courier_code)
);

CREATE TABLE shipment_subscriptions
(
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id             UUID NOT NULL REFERENCES users (id),
    shipment_id         UUID NOT NULL REFERENCES shipments (id),

    subscribed_statuses shipment_status[],

    label               TEXT,

    created_at          TIMESTAMPTZ      DEFAULT now(),
    UNIQUE (user_id, shipment_id)
);

CREATE TABLE tracking_events
(
    id                UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_id       UUID                  NOT NULL REFERENCES shipments (id) ON DELETE CASCADE,
    raw_status        TEXT                  NOT NULL,
    normalized_status shipment_status       NOT NULL,
    description       TEXT                  NOT NULL,
    location          TEXT,
    occurred_at       TIMESTAMPTZ           NOT NULL,
    source            tracking_event_source NOT NULL,
    created_at        TIMESTAMPTZ      DEFAULT now()
);


CREATE TABLE tracking_jobs
(
    shipment_id      UUID PRIMARY KEY REFERENCES shipments (id) ON DELETE CASCADE,
    next_run_at      TIMESTAMPTZ NOT NULL,
    interval_minutes INT         NOT NULL DEFAULT 30,
    attempt          INT         NOT NULL DEFAULT 0,
    is_active        BOOLEAN     NOT NULL DEFAULT true,
    updated_at       TIMESTAMPTZ          DEFAULT now()
);


CREATE TABLE status_mappings
(
    id                UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    courier_code      TEXT            NOT NULL,
    raw_status        TEXT            NOT NULL,
    normalized_status shipment_status NOT NULL,
    UNIQUE (courier_code, raw_status)
);



CREATE TABLE notification_logs
(
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    shipment_id     UUID REFERENCES shipments (id),
    event_id        UUID REFERENCES tracking_events (id),
    channel         notification_channel NOT NULL,
    recipient_to    TEXT                 NOT NULL,
    message_content TEXT                 NOT NULL,
    status          TEXT             DEFAULT 'PENDING',
    error_message   TEXT,
    sent_at         TIMESTAMPTZ,
    created_at      TIMESTAMPTZ      DEFAULT now()
);



CREATE TABLE webhook_logs
(
    id           UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    payload      JSONB NOT NULL,
    processed_at TIMESTAMPTZ,
    created_at   TIMESTAMPTZ      DEFAULT now()
);

ALTER TABLE shipments
    ADD COLUMN updated_at TIMESTAMPTZ DEFAULT now();
ALTER TABLE shipment_subscriptions
    ADD COLUMN updated_at TIMESTAMPTZ DEFAULT now();

INSERT INTO status_mappings (courier_code, raw_status, normalized_status)
VALUES ('biteship', 'confirmed', 'CREATED'),
       ('biteship', 'allocated', 'CREATED'),
       ('biteship', 'pickingUp', 'RECEIVED'),
       ('biteship', 'picked', 'RECEIVED'),
       ('biteship', 'droppingOff', 'IN_TRANSIT'),
       ('biteship', 'onHold', 'IN_TRANSIT'),
       ('biteship', 'returnInTransit', 'RETURNED'),
       ('biteship', 'delivered', 'DELIVERED'),
       ('biteship', 'returned', 'RETURNED'),
       ('biteship', 'rejected', 'FAILED'),
       ('biteship', 'courierNotFound', 'FAILED'),
       ('biteship', 'disposed', 'FAILED'),
       ('biteship', 'cancelled', 'CANCELLED');

ALTER TABLE status_mappings RENAME COLUMN courier_code TO platform;

-- Dummy user data
INSERT INTO users (id, name, phone_number, email, created_at)
VALUES
    ('550e8400-e29b-41d4-a716-446655440000', 'John Doe', '+1234567890', 'john.doe@example.com', '2024-01-15 10:30:00+00'),
    ('550e8400-e29b-41d4-a716-446655440001', 'Jane Smith', '+1234567891', 'jane.smith@example.com', '2024-01-20 14:45:00+00');

-- User notification preferences
INSERT INTO user_notification_preferences (user_id, default_channels, updated_at)
VALUES
    ('550e8400-e29b-41d4-a716-446655440000', '{WHATSAPP,EMAIL}', '2024-01-15 10:30:00+00'),
    ('550e8400-e29b-41d4-a716-446655440001', '{EMAIL}', '2024-01-20 14:45:00+00');

-- Shipment subscriptions
INSERT INTO shipment_subscriptions (id, user_id, shipment_id, subscribed_statuses, label, created_at, updated_at)
VALUES
    ('850e8400-e29b-41d4-a716-446655440000', '550e8400-e29b-41d4-a716-446655440000', 'e975bb9f-c0b5-4fe5-a20d-e34eba31dafa', '{OUT_FOR_DELIVERY,DELIVERED,FAILED}', 'Electronics Order', '2024-01-16 09:15:00+00', '2024-01-16 09:15:00+00'),
    ('850e8400-e29b-41d4-a716-446655440001', '550e8400-e29b-41d4-a716-446655440001', 'e975bb9f-c0b5-4fe5-a20d-e34eba31dafa', '{DELIVERED,FAILED, RETURNED}', 'Books Order', '2024-01-22 16:15:00+00', '2024-01-22 16:15:00+00');
