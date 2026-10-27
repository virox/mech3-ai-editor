import defaultLayoutJson from './ai-layout.json';
import type { Json } from './types';
import { ensureTag, parseScalar, removeTag, tagArray, tags, type VehicleView } from './workspace';

export type AiControlType = 'text'|'number'|'checkbox'|'select'|'reference'|'team'|'json';
export type AiFormatType = 'auto'|'string'|'number'|'integer'|'decimal'|'boolean';
export type AiFieldWidth = 'small'|'normal'|'wide'|'full';
export type AiOptionSource = 'strategies'|'chassis'|'binary'|'scan-jump'|'none';
export type AiReferenceKind = 'valve'|'region'|'vehicle'|'network'|'none';

export interface AiLayoutOption { value:string; label:string }
export interface AiLayoutField {
  id:string;
  label:string;
  description:string;
  binding:string;
  control:AiControlType;
  format:AiFormatType;
  width:AiFieldWidth;
  readOnly?:boolean;
  precision?:number;
  optionSource?:AiOptionSource;
  referenceKind?:AiReferenceKind;
  options?:AiLayoutOption[];
}
export interface AiLayoutGroup {
  id:string;
  type:'group';
  title:string;
  description:string;
  columns:1|2|3;
  presenceTag?:string;
  fields:AiLayoutField[];
}
export interface AiLayoutRepeater {
  id:string;
  type:'repeater';
  title:string;
  description:string;
  binding:string;
  itemLength:number;
  itemLabel:string;
  addLabel:string;
  groups:AiLayoutGroup[];
}
export type AiLayoutBlock = AiLayoutGroup|AiLayoutRepeater;
export interface AiLayoutTab { id:string; title:string; description:string; blocks:AiLayoutBlock[] }
export interface AiLayout { version:1; tabs:AiLayoutTab[] }

export const CONTROL_TYPES:AiControlType[]=['text','number','checkbox','select','reference','team','json'];
export const FORMAT_TYPES:AiFormatType[]=['auto','string','number','integer','decimal','boolean'];
export const FIELD_WIDTHS:AiFieldWidth[]=['small','normal','wide','full'];
export const OPTION_SOURCES:AiOptionSource[]=['none','strategies','chassis','binary','scan-jump'];
export const REFERENCE_KINDS:AiReferenceKind[]=['none','valve','region','vehicle','network'];

export function displayBinding(binding:string):string { return binding.startsWith('tag:')?binding.slice(4):binding }

export const DEFAULT_AI_LAYOUT = validateAiLayout(defaultLayoutJson);

export function cloneAiLayout(layout:AiLayout):AiLayout { return JSON.parse(JSON.stringify(layout)) as AiLayout; }

function object(value:unknown):Record<string,unknown> {
  if(!value||typeof value!=='object'||Array.isArray(value))throw new Error('Expected an object.');
  return value as Record<string,unknown>;
}
function text(value:unknown,name:string):string { if(typeof value!=='string')throw new Error(`${name} must be text.`);return value }
function oneOf<T extends string>(value:unknown,values:readonly T[],name:string):T { if(typeof value!=='string'||!values.includes(value as T))throw new Error(`${name} must be one of: ${values.join(', ')}.`);return value as T }
function uniqueId(value:unknown,ids:Set<string>,name:string):string { const id=text(value,name).trim();if(!id)throw new Error(`${name} cannot be empty.`);if(ids.has(id))throw new Error(`Duplicate layout id: ${id}.`);ids.add(id);return id }
function positiveInteger(value:unknown,name:string,min=1,max=100):number { if(!Number.isInteger(value)||Number(value)<min||Number(value)>max)throw new Error(`${name} must be an integer from ${min} to ${max}.`);return Number(value) }

export function validateBinding(binding:string,insideRepeater:boolean):void {
  const valid=binding==='name'||binding==='record'||/^record\[\d+\](?:\[\d+\])?$/.test(binding)||/^tag:[A-Za-z_][A-Za-z0-9_]*\[\d+\]$/.test(binding)||(insideRepeater&&/^item\[\d+\]$/.test(binding));
  if(!valid)throw new Error(`Invalid field binding "${binding}".`);
}

export function validateAiLayout(value:unknown):AiLayout {
  const root=object(value);if(root.version!==1)throw new Error('AI layout version must be 1.');
  if(!Array.isArray(root.tabs)||root.tabs.length===0)throw new Error('The AI layout must contain at least one tab.');
  const ids=new Set<string>();
  const field=(raw:unknown,insideRepeater:boolean,path:string):AiLayoutField=>{
    const f=object(raw);const binding=text(f.binding,`${path}.binding`);validateBinding(binding,insideRepeater);
    const control=oneOf(f.control,CONTROL_TYPES,`${path}.control`);const format=oneOf(f.format,FORMAT_TYPES,`${path}.format`);
    if(control==='checkbox'&&format!=='boolean')throw new Error(`${path}: checkbox controls require boolean format.`);
    if(control==='team'&&binding!=='record[3]')throw new Error(`${path}: team controls must bind to record[3].`);
    if(control==='json'&&binding!=='record')throw new Error(`${path}: JSON controls must bind to record.`);
    const options=Array.isArray(f.options)?f.options.map((rawOption,index)=>{const option=object(rawOption);return {value:text(option.value,`${path}.options[${index}].value`),label:text(option.label,`${path}.options[${index}].label`)}}):undefined;
    const precision=f.precision===undefined?undefined:positiveInteger(f.precision,`${path}.precision`,0,12);
    return {id:uniqueId(f.id,ids,`${path}.id`),label:text(f.label,`${path}.label`),description:text(f.description??'',`${path}.description`),binding,control,format,width:oneOf(f.width??'normal',FIELD_WIDTHS,`${path}.width`),readOnly:f.readOnly===true,precision,optionSource:f.optionSource===undefined?undefined:oneOf(f.optionSource,OPTION_SOURCES,`${path}.optionSource`),referenceKind:f.referenceKind===undefined?undefined:oneOf(f.referenceKind,REFERENCE_KINDS,`${path}.referenceKind`),options};
  };
  const group=(raw:unknown,insideRepeater:boolean,path:string):AiLayoutGroup=>{
    const g=object(raw);if(g.type!=='group')throw new Error(`${path}.type must be group.`);if(!Array.isArray(g.fields))throw new Error(`${path}.fields must be an array.`);
    const presenceTag=g.presenceTag===undefined?undefined:text(g.presenceTag,`${path}.presenceTag`).trim();
    if(presenceTag&&!/^[A-Za-z_][A-Za-z0-9_]*$/.test(presenceTag))throw new Error(`${path}.presenceTag is invalid.`);
    return {id:uniqueId(g.id,ids,`${path}.id`),type:'group',title:text(g.title,`${path}.title`),description:text(g.description??'',`${path}.description`),columns:positiveInteger(g.columns??2,`${path}.columns`,1,3) as 1|2|3,presenceTag:presenceTag||undefined,fields:g.fields.map((entry,index)=>field(entry,insideRepeater,`${path}.fields[${index}]`))};
  };
  const tabs=root.tabs.map((rawTab,tabIndex)=>{
    const tab=object(rawTab);if(!Array.isArray(tab.blocks))throw new Error(`tabs[${tabIndex}].blocks must be an array.`);
    const blocks=tab.blocks.map((rawBlock,blockIndex):AiLayoutBlock=>{
      const block=object(rawBlock);const path=`tabs[${tabIndex}].blocks[${blockIndex}]`;
      if(block.type==='group')return group(block,false,path);
      if(block.type!=='repeater')throw new Error(`${path}.type must be group or repeater.`);
      const binding=text(block.binding,`${path}.binding`);if(!/^tag:[A-Za-z_][A-Za-z0-9_]*$/.test(binding))throw new Error(`${path}.binding must be a tag:list binding.`);
      if(!Array.isArray(block.groups)||block.groups.length===0)throw new Error(`${path}.groups must contain at least one group.`);
      return {id:uniqueId(block.id,ids,`${path}.id`),type:'repeater',title:text(block.title,`${path}.title`),description:text(block.description??'',`${path}.description`),binding,itemLength:positiveInteger(block.itemLength,`${path}.itemLength`),itemLabel:text(block.itemLabel,`${path}.itemLabel`),addLabel:text(block.addLabel,`${path}.addLabel`),groups:block.groups.map((entry,index)=>group(entry,true,`${path}.groups[${index}]`))};
    });
    return {id:uniqueId(tab.id,ids,`tabs[${tabIndex}].id`),title:text(tab.title,`tabs[${tabIndex}].title`),description:text(tab.description??'',`tabs[${tabIndex}].description`),blocks};
  });
  return {version:1,tabs};
}

export function layoutErrors(value:unknown):string[] { try{validateAiLayout(value);return []}catch(error){return [error instanceof Error?error.message:String(error)]} }

function bindingMatch(binding:string){return /^(record|tag:[A-Za-z_][A-Za-z0-9_]*|item)(?:\[(\d+)\])?(?:\[(\d+)\])?$/.exec(binding)}
export function fieldValue(vehicle:VehicleView,binding:string,item?:Json[]):Json|undefined {
  if(binding==='name')return vehicle.name;if(binding==='record')return vehicle.record;
  const match=bindingMatch(binding);if(!match)return undefined;const first=match[2]===undefined?undefined:Number(match[2]);const second=match[3]===undefined?undefined:Number(match[3]);
  let value:Json|undefined;
  if(match[1]==='record')value=first===undefined?vehicle.record:vehicle.record[first];
  else if(match[1]==='item')value=first===undefined?item:item?.[first];
  else {const tag=match[1].slice(4);const values=tagArray(vehicle.record,tag);value=first===undefined?values:values?.[first]}
  return second===undefined?value:(Array.isArray(value)?value[second]:undefined);
}

function formattedInput(value:Json|undefined,field:AiLayoutField):string {
  if(value==null)return '';if(field.format==='decimal'&&typeof value==='number'&&field.precision!==undefined)return value.toFixed(field.precision);return String(value);
}
export function displayValue(vehicle:VehicleView,field:AiLayoutField,item?:Json[]):string { return formattedInput(fieldValue(vehicle,field.binding,item),field) }
export function booleanValue(vehicle:VehicleView,field:AiLayoutField,item?:Json[]):boolean { const value=fieldValue(vehicle,field.binding,item);return value===true||Number(value)!==0 }

function converted(textValue:string,old:Json|undefined,field:AiLayoutField):Json {
  if(field.format==='string')return textValue;
  if(field.format==='boolean')return textValue==='true'||textValue==='1'?1:0;
  if(field.format==='number'||field.format==='decimal'){const value=Number(textValue);return Number.isFinite(value)?value:(old??0)}
  if(field.format==='integer'){const value=Number(textValue);return Number.isFinite(value)?Math.trunc(value):(old??0)}
  return parseScalar(textValue,old);
}
export function setFieldValue(vehicle:VehicleView,field:AiLayoutField,textValue:string,item:Json[]|undefined,setName:(name:string)=>void):void {
  if(field.binding==='name'){setName(textValue);return}if(field.binding==='record')return;
  const match=bindingMatch(field.binding);if(!match)return;const first=match[2]===undefined?undefined:Number(match[2]);const second=match[3]===undefined?undefined:Number(match[3]);if(first===undefined)return;
  let owner:Json[];
  if(match[1]==='record')owner=vehicle.record;
  else if(match[1]==='item'){if(!item)return;owner=item}
  else owner=ensureTag(vehicle.record,match[1].slice(4),first+1);
  while(owner.length<=first)owner.push(0);
  if(second===undefined)owner[first]=converted(textValue,owner[first],field);
  else {let nested=Array.isArray(owner[first])?owner[first] as Json[]:[];while(nested.length<=second)nested.push(0);nested[second]=converted(textValue,nested[second],field);owner[first]=nested}
}

export function groupPresent(vehicle:VehicleView,group:AiLayoutGroup):boolean { return !group.presenceTag||!!tagArray(vehicle.record,group.presenceTag) }
export function setGroupPresent(vehicle:VehicleView,group:AiLayoutGroup,present:boolean):void {
  const presenceTag=group.presenceTag;if(!presenceTag)return;if(!present){removeTag(vehicle.record,presenceTag);return}
  const largest=group.fields.map(field=>field.binding.match(new RegExp(`^tag:${presenceTag.replace(/[.*+?^${}()|[\]\\]/g,'\\$&')}\\[(\\d+)\\]$`))?.[1]).filter((value):value is string=>value!==undefined).reduce((max,value)=>Math.max(max,Number(value)),-1);
  ensureTag(vehicle.record,presenceTag,largest+1);
}
export function repeaterItems(vehicle:VehicleView,repeater:AiLayoutRepeater):Json[] { const tag=repeater.binding.slice(4);return tagArray(vehicle.record,tag)??[] }
export function addRepeaterItem(vehicle:VehicleView,repeater:AiLayoutRepeater):void { ensureTag(vehicle.record,repeater.binding.slice(4),0).push(Array(repeater.itemLength).fill(0)) }
export function removeRepeaterItem(vehicle:VehicleView,repeater:AiLayoutRepeater,index:number):void { repeaterItems(vehicle,repeater).splice(index,1) }

export function allLayoutFields(layout:AiLayout):AiLayoutField[] { return layout.tabs.flatMap(tab=>tab.blocks.flatMap(block=>block.type==='group'?block.fields:block.groups.flatMap(group=>group.fields))) }
export function newLayoutId(prefix:string):string { return `${prefix}-${crypto.randomUUID()}` }
export function tagValue(vehicle:VehicleView,name:string):Json|undefined { return tags(vehicle.record).find(entry=>entry.tag===name)?.value }
