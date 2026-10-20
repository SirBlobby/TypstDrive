export interface PickerCharacter {
    character: string;
    name: string;
}

export interface PickerCategory {
    name: string;
    characters: PickerCharacter[];
}

function category(name: string, entries: [string, string][]): PickerCategory {
    return { name, characters: entries.map(([character, characterName]) => ({ character, name: characterName })) };
}

export const symbolCategories: PickerCategory[] = [
    category('Greek', [
        ['α', 'alpha'], ['β', 'beta'], ['γ', 'gamma'], ['δ', 'delta'], ['ε', 'epsilon'], ['ζ', 'zeta'],
        ['η', 'eta'], ['θ', 'theta'], ['ι', 'iota'], ['κ', 'kappa'], ['λ', 'lambda'], ['μ', 'mu'],
        ['ν', 'nu'], ['ξ', 'xi'], ['π', 'pi'], ['ρ', 'rho'], ['σ', 'sigma'], ['τ', 'tau'],
        ['υ', 'upsilon'], ['φ', 'phi'], ['χ', 'chi'], ['ψ', 'psi'], ['ω', 'omega'],
        ['Γ', 'Gamma'], ['Δ', 'Delta'], ['Θ', 'Theta'], ['Λ', 'Lambda'], ['Ξ', 'Xi'], ['Π', 'Pi'],
        ['Σ', 'Sigma'], ['Φ', 'Phi'], ['Ψ', 'Psi'], ['Ω', 'Omega']
    ]),
    category('Arrows', [
        ['→', 'arrow right'], ['←', 'arrow left'], ['↑', 'arrow up'], ['↓', 'arrow down'], ['↔', 'arrow left right'],
        ['↕', 'arrow up down'], ['⇒', 'double arrow right implies'], ['⇐', 'double arrow left'], ['⇔', 'double arrow left right iff'],
        ['↦', 'maps to'], ['↗', 'arrow north east'], ['↘', 'arrow south east'], ['↙', 'arrow south west'], ['↖', 'arrow north west'],
        ['⟶', 'long arrow right'], ['⟵', 'long arrow left'], ['↩', 'arrow hook left'], ['↪', 'arrow hook right']
    ]),
    category('Math', [
        ['±', 'plus minus'], ['∓', 'minus plus'], ['×', 'times multiply'], ['÷', 'divide'], ['·', 'dot'], ['∘', 'compose circle'],
        ['√', 'square root'], ['∞', 'infinity'], ['∂', 'partial'], ['∇', 'nabla'], ['∫', 'integral'], ['∬', 'double integral'],
        ['∮', 'contour integral'], ['∑', 'sum'], ['∏', 'product'], ['∆', 'increment'], ['′', 'prime'], ['°', 'degree'],
        ['‰', 'per mille'], ['∝', 'proportional']
    ]),
    category('Relations', [
        ['≠', 'not equal'], ['≈', 'approximately'], ['≡', 'equivalent'], ['≤', 'less or equal'], ['≥', 'greater or equal'],
        ['≪', 'much less'], ['≫', 'much greater'], ['∼', 'similar tilde'], ['≅', 'congruent'], ['≔', 'colon equals define'],
        ['⊥', 'perpendicular'], ['∥', 'parallel']
    ]),
    category('Sets and logic', [
        ['∈', 'element of in'], ['∉', 'not element of'], ['⊂', 'subset'], ['⊃', 'superset'], ['⊆', 'subset or equal'],
        ['⊇', 'superset or equal'], ['∪', 'union'], ['∩', 'intersection'], ['∅', 'empty set'], ['ℕ', 'natural numbers'],
        ['ℤ', 'integers'], ['ℚ', 'rationals'], ['ℝ', 'reals'], ['ℂ', 'complex'], ['∀', 'for all'], ['∃', 'exists'],
        ['∄', 'not exists'], ['¬', 'not'], ['∧', 'and'], ['∨', 'or'], ['⊕', 'xor plus circle'], ['⊤', 'top true'], ['∴', 'therefore'],
        ['∵', 'because']
    ]),
    category('Punctuation', [
        ['—', 'em dash'], ['–', 'en dash'], ['…', 'ellipsis'], ['•', 'bullet'], ['‣', 'triangle bullet'], ['§', 'section'],
        ['¶', 'pilcrow paragraph'], ['†', 'dagger'], ['‡', 'double dagger'], ['«', 'guillemet left'], ['»', 'guillemet right'],
        ['‹', 'single guillemet left'], ['›', 'single guillemet right'], ['¡', 'inverted exclamation'], ['¿', 'inverted question']
    ]),
    category('Currency and marks', [
        ['€', 'euro'], ['£', 'pound'], ['¥', 'yen'], ['¢', 'cent'], ['₹', 'rupee'], ['₩', 'won'], ['₿', 'bitcoin'],
        ['©', 'copyright'], ['®', 'registered'], ['™', 'trademark'], ['℃', 'celsius'], ['℉', 'fahrenheit'], ['№', 'numero'],
        ['✓', 'check'], ['✗', 'cross ballot'], ['★', 'star filled'], ['☆', 'star outline'], ['♠', 'spade'], ['♥', 'heart suit'],
        ['♦', 'diamond suit'], ['♣', 'club']
    ])
];
