import { mount } from 'svelte';
import Studio from './Studio.svelte';
import './app.css';
import './studio.css';
import './studio-extra.css';
mount(Studio, { target: document.getElementById('app')! });
