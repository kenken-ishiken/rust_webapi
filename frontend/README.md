# Next.js Frontend for Product List

This is a simple Next.js frontend application that displays product information from the Rust WebAPI backend.

## Features

- Displays a list of products with names and descriptions
- Responsive design using Tailwind CSS
- Static export configuration for easy deployment
- No authentication or CRUD operations

## Setup

1. Install dependencies:

```bash
cd frontend
npm install
```

2. Run the development server:

```bash
npm run dev
```

3. Open [http://localhost:3000](http://localhost:3000) in your browser to see the application.

## Build

To build the application for production:

```bash
npm run build
```

This will create a static export in the `out` directory that can be served by any static file server.

## Project Structure

- `src/app/page.tsx`: Main page component
- `src/app/components/ProductList.tsx`: Component for displaying the product list
- `src/app/layout.tsx`: Root layout component
- `src/app/globals.css`: Global CSS styles
