-- Performance optimization indexes

-- Products table indexes
CREATE INDEX IF NOT EXISTS idx_products_category_id ON products(category_id);
CREATE INDEX IF NOT EXISTS idx_products_is_active ON products(is_active);
CREATE INDEX IF NOT EXISTS idx_products_created_at ON products(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_products_name_gin ON products USING gin(to_tsvector('english', name));

-- Items table indexes
CREATE INDEX IF NOT EXISTS idx_items_is_deleted ON items(is_deleted);
CREATE INDEX IF NOT EXISTS idx_items_created_at ON items(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_items_updated_at ON items(updated_at DESC);

-- Categories table indexes
CREATE INDEX IF NOT EXISTS idx_categories_parent_id ON categories(parent_id);
CREATE INDEX IF NOT EXISTS idx_categories_is_active ON categories(is_active);
CREATE INDEX IF NOT EXISTS idx_categories_path ON categories(path);

-- Product_prices table indexes
CREATE INDEX IF NOT EXISTS idx_product_prices_product_id ON product_prices(product_id);
CREATE INDEX IF NOT EXISTS idx_product_prices_effective_date ON product_prices(effective_date DESC);

-- Product_inventory table indexes
CREATE INDEX IF NOT EXISTS idx_product_inventory_product_id ON product_inventory(product_id);
CREATE INDEX IF NOT EXISTS idx_product_inventory_location ON product_inventory(location);

-- Composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_products_category_active ON products(category_id, is_active);
CREATE INDEX IF NOT EXISTS idx_items_deleted_created ON items(is_deleted, created_at DESC);

-- Partial indexes for performance
CREATE INDEX IF NOT EXISTS idx_products_active_only ON products(id) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_items_not_deleted ON items(id) WHERE is_deleted = false;

-- Add statistics for query optimization
ANALYZE products;
ANALYZE items;
ANALYZE categories;
ANALYZE product_prices;
ANALYZE product_inventory;