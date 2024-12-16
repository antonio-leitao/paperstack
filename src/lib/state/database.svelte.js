import { load } from '@tauri-apps/plugin-store';

function generate_unique_id() {
    // Get current Unix timestamp in milliseconds
    const timestamp = Date.now();
    // Generate a random number between 1000 and 9999
    const randomNum = Math.floor(Math.random() * 9000) + 1000;
    return parseInt(`${timestamp}${randomNum}`);
}

let store = $state(null);
let papers = $state([]);
let stackID = $state(null);
export let Stack = {
    get stackID() {
        return stackID;
    },
    set stackID(val) {
        stackID = val;
    },
    get papers() {
        return papers;
    },
    set papers(val) {
        papers = val;
    },
}


export async function initializeStore(params) {
    if (!store) {
        store = await load('store.json', { autoSave: true });
    }
    console.log("initializeStore", params);
    Stack.stackID = "files"
    Stack.papers = await store.get('files') ?? [];
    return;
}

export async function createFile(file){
    if (!store) throw new Error('Store not initialized');
    try {
        const files = await store.get('files') ?? [];
        let id = generate_unique_id();
        await store.set('files', [...files, {...file, id}]);
		Stack.papers =  [...files, {...file, id}]
        return
    } catch (error) {
        console.error('Error creating file in store:', error);
        throw error;
    }
}

export async function updateFile(update_id,updatedFile) {
    if (!store) throw new Error('Store not initialized');
    try {
        const files = await store.get('files') ?? [];
        const updatedFiles = files.map(file => 
            file.id === update_id ? { ...file, ...updatedFile } : file
        );
        await store.set('files', updatedFiles);
		Stack.papers =  updatedFiles
        return
    } catch (error) {
        console.error('Error updating file in store:', error);
        throw error;
    }
}

export async function deleteFile(fileId) {
    if (!store) throw new Error('Store not initialized');
    try {
        const files = await store.get('files') ?? [];
        const filteredFiles = files.filter(file => file.id !== fileId);
        await store.set('files', filteredFiles);
		Stack.papers = filteredFiles;
        return;
    } catch (error) {
        console.error('Error deleting file from store:', error);
        throw error;
    }
}