import { createClient } from '@hey-api/openapi-fetch';

// Place name service using Norwegian Mapping Authority API
export class PlaceService {
  private client = createClient({
    baseUrl: 'https://api.kartverket.no/stedsnavn/v1',
  });

  // Search for places by name - uses /navn endpoint
  async searchByName(name: string): Promise<any> {
    const response = await this.client.GET('/navn', {
      params: {
        query: { sok: name }
      }
    });
    return response.data;
  }

  // Get places near a geographic point - uses /punkt endpoint
  async searchNearPoint(lat: number, lon: number, radius: number = 1000): Promise<any> {
    const response = await this.client.GET('/punkt', {
      params: {
        query: {
          nord: lat,
          ost: lon,
          radius: radius
        }
      }
    });
    return response.data;
  }

  // Search for a specific place - uses /sted endpoint
  async searchPlace(query: string): Promise<any> {
    const response = await this.client.GET('/sted', {
      params: {
        query: { sok: query }
      }
    });
    return response.data;
  }

  // Get available languages - uses /sprak endpoint
  async getLanguages(): Promise<any> {
    const response = await this.client.GET('/sprak');
    return response.data;
  }

  // Get name object types - uses /navneobjekttyper endpoint
  async getNameObjectTypes(): Promise<any> {
    const response = await this.client.GET('/navneobjekttyper');
    return response.data;
  }
}