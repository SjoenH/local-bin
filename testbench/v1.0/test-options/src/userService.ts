// User service with endpoint usage
import { api } from './client';

export class UserService {
  async getUsers() {
    return api.GET('/api/users');
  }

  async createUser(userData: any) {
    return api.POST('/api/users', { body: userData });
  }

  async getUserById(id: string) {
    return api.GET('/api/users/{id}', { params: { path: { id } } });
  }

  // This method exists but doesn't use the PUT endpoint
  async updateUser(id: string, userData: any) {
    // Implementation would use PUT /api/users/{id}
    console.log('Updating user', id, userData);
  }
}

// Direct API calls for testing different patterns
export async function directCalls() {
  // Used endpoints
  await api.GET('/api/users');
  await api.POST('/api/users');
  await api.GET('/api/users/{id}');

  // Unused endpoint - PUT /api/users/{id}
  // Not called here
}