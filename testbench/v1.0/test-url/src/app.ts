import { createClient } from '@hey-api/openapi-fetch';

// Create client for Norwegian Mapping Authority API
const client = createClient({
  baseUrl: 'https://api.kartverket.no/stedsnavn/v1',
});

// Utility functions that use the place service
export async function searchNorwegianPlaces() {
  // Search for places by name - uses /navn endpoint
  const nameResults = await client.GET('/navn', {
    params: {
      query: { sok: 'Oslo' }
    }
  });

  // Search near a geographic point - uses /punkt endpoint
  const pointResults = await client.GET('/punkt', {
    params: {
      query: {
        nord: 59.9139,
        ost: 10.7522,
        radius: 1000
      }
    }
  });

  // Search for a specific place - uses /sted endpoint
  const placeResults = await client.GET('/sted', {
    params: {
      query: { sok: 'Trondheim' }
    }
  });

  return {
    nameResults: nameResults.data,
    pointResults: pointResults.data,
    placeResults: placeResults.data
  };
}

export async function getPlaceMetadata() {
  // Get available languages - uses /sprak endpoint
  const languages = await client.GET('/sprak');

  // Get name object types - uses /navneobjekttyper endpoint
  const nameTypes = await client.GET('/navneobjekttyper');

  return {
    languages: languages.data,
    nameTypes: nameTypes.data
  };
}

// Direct method calls that epcheck should detect
export async function directApiCalls() {
  // These GET calls should be detected by epcheck
  const navnData = await client.GET('/navn');
  const punktData = await client.GET('/punkt');
  const stedData = await client.GET('/sted');
  const sprakData = await client.GET('/sprak');
  const typerData = await client.GET('/navneobjekttyper');

  return {
    navn: navnData.data,
    punkt: punktData.data,
    sted: stedData.data,
    sprak: sprakData.data,
    typer: typerData.data
  };
}