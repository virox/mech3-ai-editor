import { mount } from 'svelte';
import Switchboard from './Switchboard.svelte';
import './switchboard.css';
import './inline-controls.css';

mount(Switchboard, { target: document.getElementById('app')! });
