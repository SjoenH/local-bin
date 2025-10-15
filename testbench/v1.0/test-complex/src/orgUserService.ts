import { createClient } from '@hey-api/openapi-ts';

const client = createClient({
  baseUrl: 'https://api.example.com',
});

// Used endpoints with nested paths and parameters
export async function getOrgUsers(orgId: string) {
  const response = await client.GET('/api/organizations/{orgId}/users', {
    params: {
      path: { orgId },
    },
  });
  return response.data;
}

export async function createOrgUser(orgId: string, userData: any) {
  const response = await client.POST('/api/organizations/{orgId}/users', {
    params: {
      path: { orgId },
    },
    body: userData,
  });
  return response.data;
}

export async function getOrgUser(orgId: string, userId: string) {
  const response = await client.GET('/api/organizations/{orgId}/users/{userId}', {
    params: {
      path: { orgId, userId },
    },
  });
  return response.data;
}

// PUT and DELETE endpoints for /api/organizations/{orgId}/users/{userId} are unused