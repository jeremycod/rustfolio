import axios from 'axios'

export async function getHealth(): Promise<void> {
  await axios.get('/health')
}
