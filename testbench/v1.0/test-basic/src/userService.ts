import { createClient } from '@hey-api/openapi-ts';

const client = createClient({
  baseUrl: 'https://api.example.com',
});

// Used endpoints
export async function getUsers() {
  const response = await client.GET('/api/users');
  return response.data;
}

export async function createUser(userData: any) {
  const response = await client.POST('/api/users', {
    body: userData,
  });
  return response.data;
}

export async function getUserById(id: string) {
  const response = await client.GET('/api/users/{id}', {
    params: {
      path: { id },
    },
  });
  return response.data;
}

// This endpoint is defined in spec but not used in code
// PUT /api/users/{id} - unused