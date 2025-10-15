import { createClient } from '@hey-api/openapi-ts';

const client = createClient({ baseUrl: 'https://api.example.com' });

export async function getOrders() {
  return client.GET('/api/orders');
}

export async function getOrder(id: string) {
  return client.GET('/api/orders/{id}', { params: { path: { id } } });
}