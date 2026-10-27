import { mount } from 'svelte';
import LayoutDevStudio from './LayoutDevStudio.svelte';
import './app.css';
import './studio.css';
import './studio-extra.css';
mount(LayoutDevStudio,{target:document.getElementById('app')!});
