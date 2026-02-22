import type {
  DynamicSystemInfo,
  StaticSystemInfo,
  SystemInfo,
} from '@/shared/types/systemInfo'
import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'

export async function getStaticSystemInfo(): Promise<StaticSystemInfo> {
  return invoke<StaticSystemInfo>('get_static_system_info')
}

export async function getDynamicSystemInfo(): Promise<DynamicSystemInfo> {
  return invoke<DynamicSystemInfo>('get_dynamic_system_info')
}

export async function getSystemInfo(): Promise<SystemInfo> {
  return invoke<SystemInfo>('get_system_info')
}

export function useStaticSystemInfo() {
  return useQuery({
    queryKey: ['staticSystemInfo'],
    queryFn: getStaticSystemInfo,
    staleTime: Infinity,
    gcTime: Infinity,
  })
}

export function useDynamicSystemInfo() {
  return useQuery({
    queryKey: ['dynamicSystemInfo'],
    queryFn: getDynamicSystemInfo,
    refetchInterval: 1000,
  })
}

export function useSystemInfo() {
  return useQuery({
    queryKey: ['systemInfo'],
    queryFn: getSystemInfo,
    refetchInterval: 1000,
  })
}
