export interface Contract {
  id: string;
  contract_id: string;
  wasm_hash: string;
  name: string;
  description?: string;
  publisher_id: string;
  network: 'mainnet' | 'testnet' | 'futurenet';
  is_verified: boolean;
  category?: string;
  tags: string[];
  created_at: string;
  updated_at: string;
}

export interface ContractVersion {
  id: string;
  contract_id: string;
  version: string;
  wasm_hash: string;
  source_url?: string;
  commit_hash?: string;
  release_notes?: string;
  created_at: string;
}

export interface Publisher {
  id: string;
  stellar_address: string;
  username?: string;
  email?: string;
  github_url?: string;
  website?: string;
  created_at: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export interface ContractSearchParams {
  query?: string;
  network?: 'mainnet' | 'testnet' | 'futurenet';
  verified_only?: boolean;
  category?: string;
  tags?: string[];
  page?: number;
  page_size?: number;
}

export interface PublishRequest {
  contract_id: string;
  name: string;
  description?: string;
  network: 'mainnet' | 'testnet' | 'futurenet';
  category?: string;
  tags: string[];
  source_url?: string;
  publisher_address: string;
}

const apiUrl = (path: string) => path;

export const api = {
  // Contract endpoints
  async getContracts(params?: ContractSearchParams): Promise<PaginatedResponse<Contract>> {
    const queryParams = new URLSearchParams();
    if (params?.query) queryParams.append('query', params.query);
    if (params?.network) queryParams.append('network', params.network);
    if (params?.verified_only !== undefined) queryParams.append('verified_only', String(params.verified_only));
    if (params?.category) queryParams.append('category', params.category);
    if (params?.page) queryParams.append('page', String(params.page));
    if (params?.page_size) queryParams.append('page_size', String(params.page_size));

    const response = await fetch(apiUrl(`/api/contracts?${queryParams}`));
    if (!response.ok) throw new Error('Failed to fetch contracts');
    return response.json();
  },

  async getContract(id: string): Promise<Contract> {
    const response = await fetch(apiUrl(`/api/contracts/${id}`));
    if (!response.ok) throw new Error('Failed to fetch contract');
    return response.json();
  },

  async getContractVersions(id: string): Promise<ContractVersion[]> {
    const response = await fetch(apiUrl(`/api/contracts/${id}/versions`));
    if (!response.ok) throw new Error('Failed to fetch contract versions');
    return response.json();
  },

  async publishContract(data: PublishRequest): Promise<Contract> {
    const response = await fetch(apiUrl('/api/contracts'), {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(data),
    });
    if (!response.ok) throw new Error('Failed to publish contract');
    return response.json();
  },

  // Publisher endpoints
  async getPublisher(id: string): Promise<Publisher> {
    const response = await fetch(apiUrl(`/api/publishers/${id}`));
    if (!response.ok) throw new Error('Failed to fetch publisher');
    return response.json();
  },

  async getPublisherContracts(id: string): Promise<Contract[]> {
    const response = await fetch(apiUrl(`/api/publishers/${id}/contracts`));
    if (!response.ok) throw new Error('Failed to fetch publisher contracts');
    return response.json();
  },

  // Stats endpoint
  async getStats(): Promise<{ total_contracts: number; verified_contracts: number; total_publishers: number }> {
    const response = await fetch(apiUrl('/api/stats'));
    if (!response.ok) throw new Error('Failed to fetch stats');
    return response.json();
  },
};
