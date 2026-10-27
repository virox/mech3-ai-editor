import { mount } from 'svelte';
import DevSwitchboard from './DevSwitchboard.svelte';
import '../../switchboard/src/switchboard.css';
import '../../switchboard/src/inline-controls.css';
import './dev.css';

mount(DevSwitchboard,{target:document.getElementById('app')!});
