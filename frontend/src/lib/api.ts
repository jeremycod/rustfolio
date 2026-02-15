import axios from 'axios'

export async function getHealth(): Promise<void> {
  await axios.get('/health')
}

// In development, use empty baseURL to go through Vite proxy
// In production, use the configured base URL or default to empty (same origin)
export const api = axios.create({
    baseURL: import.meta.env.VITE_API_BASE_URL ?? "",
});