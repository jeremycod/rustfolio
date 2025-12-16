import axios from 'axios'

export async function getHealth(): Promise<void> {
  await axios.get('/health')
}
export const api = axios.create({
    baseURL: import.meta.env.VITE_API_BASE_URL ?? "http://localhost:3000",
});