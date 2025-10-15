import { createClient } from '@hey-api/openapi-ts';

const client = createClient({ baseUrl: 'https://api.example.com' });

export async function getUsers() {
  return client.GET('/api/users');
}

export async function getUser(id: string) {
  return client.GET('/api/users/{id}', { params: { path: { id } } });
}