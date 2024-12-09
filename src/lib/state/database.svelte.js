function generate_unique_id() {
    // Get current Unix timestamp in milliseconds
    const timestamp = Date.now();
    // Generate a random number between 1000 and 9999
    const randomNum = Math.floor(Math.random() * 9000) + 1000;
    return parseInt(`${timestamp}${randomNum}`);
}

let store = $state(null);

export async function initializeStore(storeInstance) {
    store = storeInstance;
}

export async function createFile(file){
    if (!store) throw new Error('Store not initialized');
    try {
        const files = await store.get('files') ?? [];
        let id = generate_unique_id();
        await store.set('files', [...files, {...file, id}]);
        return [...files, {...file, id}];
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
        return updatedFiles;
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
        return filteredFiles;
    } catch (error) {
        console.error('Error deleting file from store:', error);
        throw error;
    }
}