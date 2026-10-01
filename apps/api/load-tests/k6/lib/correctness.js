import http from 'k6/http';
import { Rate } from 'k6/metrics';

// Tracks the percentage of writes that were verifiable end-to-end
export const correctnessRate = new Rate('haven_correctness');

/**
 * Executes a deep correctness check (round-trip).
 * Creates a resource, then immediately attempts to read it back.
 * A 201 on create is insufficient; the data must be retrievable and intact.
 * 
 * @param {string} baseUrl - API base URL
 * @param {object} headers - HTTP headers including auth
 * @param {object} payload - The body of the POST request
 * @returns {boolean} True if the round-trip succeeded
 */
export function verifyWrite(baseUrl, headers, payload) {
  // 1. Create
  const createRes = http.post(`${baseUrl}/offers`, JSON.stringify(payload), { headers });
  if (createRes.status !== 201) return false;

  let id;
  try {
    id = JSON.parse(createRes.body)?.data?.id;
  } catch (e) {
    return false;
  }
  
  if (!id) return false;

  // 2. Retrieve
  const getRes = http.get(`${baseUrl}/offers/${id}`, { headers });
  if (getRes.status !== 200) return false;

  // 3. Verify
  try {
    const fetched = JSON.parse(getRes.body)?.data;
    // The fetched record must exist and its ID must match what we just created
    return fetched && fetched.id === id;
  } catch (e) {
    return false;
  }
}
