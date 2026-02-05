# Playbook Variables Editor Design

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a Variables Editor panel to the playbook editor sidebar for defining playbook input variables.

**Architecture:** Collapsible panel in left sidebar below the step toolbox, with inline editing for each variable.

**Tech Stack:** React, TypeScript, Tailwind CSS

---

## Component Architecture

**New Components:**
1. `PlaybookVariablesPanel.tsx` - The variables editor panel with add/edit/delete
2. `PlaybookSidebar.tsx` - Wrapper combining toolbox + variables panel

**Data Flow:**
- `AutomationPlaybookEditorPage` already has `variables` state
- Pass `variables` and `setVariables` to `PlaybookSidebar` → `PlaybookVariablesPanel`
- Variables saved with playbook on Save button click

**Variable Interface (existing):**
```typescript
interface Variable {
  name: string;        // e.g., "target_environment"
  var_type: string;    // "string" | "number" | "boolean"
  required: boolean;   // Must be provided when running
  default_value?: string;
}
```

---

## UI Layout

```
┌─────────────────────────────┐
│ STEPS                    [v]│  <- Collapsible header
│ ├─ Integration Step         │
│ ├─ Webhook Step             │
│ ├─ Wait Step                │
│ ├─ Condition Step           │
│ └─ Approval Step            │
├─────────────────────────────┤
│ VARIABLES            [+ Add]│  <- Collapsible header with add button
│ ┌─────────────────────────┐ │
│ │ target_env              │ │
│ │ string • required       │ │
│ │ default: "staging"   [×]│ │
│ └─────────────────────────┘ │
│ ┌─────────────────────────┐ │
│ │ retry_count             │ │
│ │ number • optional       │ │
│ │ default: 3           [×]│ │
│ └─────────────────────────┘ │
└─────────────────────────────┘
```

---

## Interactions

1. **Add Variable**: Click "+ Add" → inserts new row with name="", type="string", required=false
2. **Edit Name**: Click name text → inline text input
3. **Change Type**: Click type badge → dropdown (string/number/boolean)
4. **Toggle Required**: Click required/optional badge → toggles state
5. **Edit Default**: Click default value → inline input (type-appropriate)
6. **Delete**: Click × button → removes variable immediately

---

## Validation

- Variable names must be unique (show red border on duplicate)
- Names must be valid identifiers: `/^[a-zA-Z_][a-zA-Z0-9_]*$/`
- Empty names show validation error

---

## Implementation Tasks

### Task 1: Create PlaybookVariablesPanel component

**Files:**
- Create: `packages/frontend/web-app/src/components/playbooks/PlaybookVariablesPanel.tsx`

**Implementation:**
- Props: `variables: Variable[]`, `onChange: (variables: Variable[]) => void`
- Collapsible section with header "VARIABLES" and "+ Add" button
- List of variable cards with inline editing
- Type dropdown, required toggle, default value input, delete button
- Validation for name uniqueness and format

### Task 2: Create PlaybookSidebar wrapper component

**Files:**
- Create: `packages/frontend/web-app/src/components/playbooks/PlaybookSidebar.tsx`

**Implementation:**
- Props: `variables: Variable[]`, `onVariablesChange: (variables: Variable[]) => void`
- Renders PlaybookToolbox (existing) and PlaybookVariablesPanel
- Both sections collapsible

### Task 3: Integrate sidebar into editor page

**Files:**
- Modify: `packages/frontend/web-app/src/pages/AutomationPlaybookEditorPage.tsx`

**Changes:**
- Replace `<PlaybookToolbox />` with `<PlaybookSidebar variables={variables} onVariablesChange={setVariables} />`
- Import the new PlaybookSidebar component

### Task 4: Build and verify

**Commands:**
```bash
cd packages/frontend/web-app && npm run build
```

**Verification:**
- No TypeScript errors
- Variables panel renders in editor
- Can add, edit, delete variables
- Variables persist on save
