import { createClient } from '@hey-api/openapi-ts';

const client = createClient({ baseUrl: 'https://api.example.com' });

export async function getProducts() {
  return client.GET('/api/products');
}

export async function getProduct(id: string) {
  return client.GET('/api/products/{id}', { params: { path: { id } } });
}