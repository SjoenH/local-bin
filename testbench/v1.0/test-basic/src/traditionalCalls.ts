// Traditional HTTP call patterns
export class ApiClient {
  private baseUrl = 'https://api.example.com';

  async get(endpoint: string) {
    // This would use fetch or axios in real code
    return fetch(`${this.baseUrl}${endpoint}`);
  }

  async post(endpoint: string, data: any) {
    return fetch(`${this.baseUrl}${endpoint}`, {
      method: 'POST',
      body: JSON.stringify(data),
    });
  }
}

const api = new ApiClient();

// Used endpoints with traditional patterns
export async function fetchUsers() {
  return api.get('/api/users');
}

export async function createNewUser(userData: any) {
  return api.post('/api/users', userData);
}

// /api/products endpoint is defined but not used anywhere