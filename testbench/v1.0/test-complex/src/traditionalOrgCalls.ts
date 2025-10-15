// Traditional HTTP calls for complex endpoints
export class OrgApiClient {
  private baseUrl = 'https://api.example.com';

  async get(endpoint: string) {
    return fetch(`${this.baseUrl}${endpoint}`);
  }

  async post(endpoint: string, data: any) {
    return fetch(`${this.baseUrl}${endpoint}`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }

  async put(endpoint: string, data: any) {
    return fetch(`${this.baseUrl}${endpoint}`, {
      method: 'PUT',
      body: JSON.stringify(data),
    });
  }

  async delete(endpoint: string) {
    return fetch(`${this.baseUrl}${endpoint}`, {
      method: 'DELETE',
    });
  }
}

const orgApi = new OrgApiClient();

// Traditional call patterns
export async function fetchOrgUsersTraditional(orgId: string) {
  return orgApi.get(`/api/organizations/${orgId}/users`);
}

export async function updateOrgUserTraditional(orgId: string, userId: string, data: any) {
  return orgApi.put(`/api/organizations/${orgId}/users/${userId}`, data);
}

// /api/projects/{projectId}/tasks endpoint is defined but not used