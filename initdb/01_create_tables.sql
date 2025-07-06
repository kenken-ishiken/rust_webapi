CREATE TABLE items (
    id BIGINT PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    deleted BOOLEAN NOT NULL DEFAULT false,
    deleted_at TIMESTAMP WITH TIME ZONE
);

CREATE TABLE users (
    id BIGINT PRIMARY KEY,
    username VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL
);

CREATE TABLE categories (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    parent_id VARCHAR(255),
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (parent_id) REFERENCES categories(id) ON DELETE CASCADE,
    CONSTRAINT check_sort_order_non_negative CHECK (sort_order >= 0),
    CONSTRAINT check_name_not_empty CHECK (LENGTH(TRIM(name)) > 0),
    UNIQUE(name, parent_id)
);

-- Indexes for better performance
CREATE INDEX idx_categories_parent_id ON categories(parent_id);
CREATE INDEX idx_categories_sort_order ON categories(sort_order);
CREATE INDEX idx_categories_is_active ON categories(is_active);
CREATE INDEX idx_categories_parent_sort ON categories(parent_id, sort_order);

-- Trigger to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_categories_updated_at 
    BEFORE UPDATE ON categories 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Product tables for comprehensive product management

CREATE TABLE products (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    sku VARCHAR(50) NOT NULL UNIQUE,
    brand VARCHAR(100),
    status VARCHAR(20) NOT NULL DEFAULT 'Draft',
    category_id VARCHAR(255),
    width DECIMAL(10,1),
    height DECIMAL(10,1),
    depth DECIMAL(10,1),
    weight DECIMAL(10,1),
    shipping_class VARCHAR(50) DEFAULT 'standard',
    free_shipping BOOLEAN DEFAULT false,
    shipping_fee DECIMAL(10,2) DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL,
    CONSTRAINT check_name_not_empty CHECK (LENGTH(TRIM(name)) > 0),
    CONSTRAINT check_sku_not_empty CHECK (LENGTH(TRIM(sku)) > 0),
    CONSTRAINT check_status_valid CHECK (status IN ('Active', 'Inactive', 'Draft', 'Discontinued')),
    CONSTRAINT check_dimensions_positive CHECK (
        (width IS NULL OR width > 0) AND 
        (height IS NULL OR height > 0) AND 
        (depth IS NULL OR depth > 0)
    ),
    CONSTRAINT check_weight_positive CHECK (weight IS NULL OR weight > 0),
    CONSTRAINT check_shipping_fee_non_negative CHECK (shipping_fee >= 0)
);

CREATE TABLE product_prices (
    id BIGSERIAL PRIMARY KEY,
    product_id VARCHAR(255) NOT NULL,
    selling_price DECIMAL(12,2) NOT NULL,
    list_price DECIMAL(12,2),
    discount_price DECIMAL(12,2),
    currency VARCHAR(3) NOT NULL DEFAULT 'JPY',
    tax_included BOOLEAN NOT NULL DEFAULT true,
    effective_from TIMESTAMP WITH TIME ZONE,
    effective_until TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    CONSTRAINT check_selling_price_positive CHECK (selling_price > 0),
    CONSTRAINT check_list_price_positive CHECK (list_price IS NULL OR list_price > 0),
    CONSTRAINT check_discount_price_positive CHECK (discount_price IS NULL OR discount_price > 0),
    CONSTRAINT check_price_relationship CHECK (
        (list_price IS NULL OR selling_price <= list_price) AND
        (discount_price IS NULL OR discount_price <= selling_price)
    ),
    CONSTRAINT check_effective_dates CHECK (
        effective_from IS NULL OR effective_until IS NULL OR effective_from <= effective_until
    )
);

CREATE TABLE product_inventory (
    id BIGSERIAL PRIMARY KEY,
    product_id VARCHAR(255) NOT NULL UNIQUE,
    quantity INTEGER NOT NULL DEFAULT 0,
    reserved_quantity INTEGER NOT NULL DEFAULT 0,
    alert_threshold INTEGER,
    track_inventory BOOLEAN NOT NULL DEFAULT true,
    allow_backorder BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    CONSTRAINT check_quantity_non_negative CHECK (quantity >= 0),
    CONSTRAINT check_reserved_quantity_non_negative CHECK (reserved_quantity >= 0),
    CONSTRAINT check_reserved_not_exceeds_quantity CHECK (reserved_quantity <= quantity),
    CONSTRAINT check_alert_threshold_non_negative CHECK (alert_threshold IS NULL OR alert_threshold >= 0)
);

CREATE TABLE product_images (
    id VARCHAR(255) PRIMARY KEY,
    product_id VARCHAR(255) NOT NULL,
    url TEXT NOT NULL,
    alt_text TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_main BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    CONSTRAINT check_url_not_empty CHECK (LENGTH(TRIM(url)) > 0),
    CONSTRAINT check_sort_order_non_negative CHECK (sort_order >= 0)
);

CREATE TABLE product_tags (
    id BIGSERIAL PRIMARY KEY,
    product_id VARCHAR(255) NOT NULL,
    tag VARCHAR(100) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    UNIQUE(product_id, tag)
);

CREATE TABLE product_attributes (
    id BIGSERIAL PRIMARY KEY,
    product_id VARCHAR(255) NOT NULL,
    attribute_name VARCHAR(100) NOT NULL,
    attribute_value TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE,
    UNIQUE(product_id, attribute_name)
);

CREATE TABLE product_history (
    id BIGSERIAL PRIMARY KEY,
    product_id VARCHAR(255) NOT NULL,
    field_name VARCHAR(100) NOT NULL,
    old_value TEXT,
    new_value TEXT,
    changed_by VARCHAR(255),
    reason TEXT,
    changed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (product_id) REFERENCES products(id) ON DELETE CASCADE
);

-- Indexes for better performance
CREATE INDEX idx_products_sku ON products(sku);
CREATE INDEX idx_products_category_id ON products(category_id);
CREATE INDEX idx_products_status ON products(status);
CREATE INDEX idx_products_brand ON products(brand);
CREATE INDEX idx_products_created_at ON products(created_at);

CREATE INDEX idx_product_prices_product_id ON product_prices(product_id);
CREATE INDEX idx_product_prices_effective_dates ON product_prices(effective_from, effective_until);

CREATE INDEX idx_product_inventory_product_id ON product_inventory(product_id);
CREATE INDEX idx_product_inventory_quantity ON product_inventory(quantity);

CREATE INDEX idx_product_images_product_id ON product_images(product_id);
CREATE INDEX idx_product_images_sort_order ON product_images(sort_order);
CREATE INDEX idx_product_images_is_main ON product_images(is_main);

CREATE INDEX idx_product_tags_product_id ON product_tags(product_id);
CREATE INDEX idx_product_tags_tag ON product_tags(tag);

CREATE INDEX idx_product_attributes_product_id ON product_attributes(product_id);
CREATE INDEX idx_product_attributes_name ON product_attributes(attribute_name);

CREATE INDEX idx_product_history_product_id ON product_history(product_id);
CREATE INDEX idx_product_history_changed_at ON product_history(changed_at);
CREATE INDEX idx_product_history_field_name ON product_history(field_name);

-- Additional triggers for updated_at columns
CREATE TRIGGER update_products_updated_at 
    BEFORE UPDATE ON products 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_product_prices_updated_at 
    BEFORE UPDATE ON product_prices 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_product_inventory_updated_at 
    BEFORE UPDATE ON product_inventory 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_product_images_updated_at 
    BEFORE UPDATE ON product_images 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_product_attributes_updated_at 
    BEFORE UPDATE ON product_attributes 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Constraint to ensure only one main image per product
CREATE UNIQUE INDEX idx_product_images_main_unique 
    ON product_images(product_id) 
    WHERE is_main = true;