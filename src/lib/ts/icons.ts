import { addCollection } from '@iconify/svelte/dist/offline-functions';
import collections from 'virtual:used-icons';

for (const collection of collections) addCollection(collection);
