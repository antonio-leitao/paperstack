import { load as loadStore } from '@tauri-apps/plugin-store';
import { initializeStore } from '$lib/state/database.svelte';
import { exists, BaseDirectory } from '@tauri-apps/plugin-fs';

export const ssr = false; // Disable SSR for this page

export async function load() {
    let store_exists = await exists('store.json', { baseDir: BaseDirectory.AppData });
    if (store_exists) {
        console.log('store.json exists');
    } else {
        console.log('store.json does not exist');
    }
    const store = await loadStore('store.json', { autoSave: true });
    initializeStore(store);
    const files = await store.get('files');
    return {
        files: files ?? []
    };
} 