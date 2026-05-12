import React, { useState, useCallback, useMemo } from 'react';
import { Search, Plus, Mail, Calendar, Link as LinkIcon, MessageCircle, TrendingDown, TrendingUp } from 'lucide-react';

interface Person {
  id: string;
  name: string;
  strength: number;
  strength_label: 'ACTIVE' | 'WARM' | 'FADING';
  last_contact_at: string;
  notes?: string;
  email?: string;
  phone?: string;
}

interface Memory {
  id: string;
  person_id: string;
  content: string;
  created_at: string;
  context?: string;
}

export const HermesRolodexUI: React.FC = () => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedPerson, setSelectedPerson] = useState<Person | null>(null);
  const [people, setPeople] = useState<Person[]>([]);
  const [memories, setMemories] = useState<Memory[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [activeTab, setActiveTab] = useState<'search' | 'add' | 'upcoming'>('search');
  const [newPersonName, setNewPersonName] = useState('');
  const [newPersonEmail, setNewPersonEmail] = useState('');

  const handleSearch = useCallback(async (query: string) => {
    if (!query.trim()) {
      setPeople([]);
      return;
    }

    setIsLoading(true);
    try {
      // Call MCP tool: fuzzy_recall
      const response = await fetch('/api/mcp/call', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          tool: 'fuzzy_recall',
          input: { query },
        }),
      });
      const result = await response.json();
      setPeople(result.matches || []);
    } catch (error) {
      console.error('Search error:', error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const handleSelectPerson = useCallback(async (person: Person) => {
    setSelectedPerson(person);
    setIsLoading(true);
    try {
      // Fetch memories for this person
      const response = await fetch('/api/mcp/call', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          tool: 'get_person',
          input: { person_id_or_name: person.name },
        }),
      });
      const result = await response.json();
      setMemories(result.memories || []);
    } catch (error) {
      console.error('Error fetching person details:', error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const handleAddPerson = useCallback(async () => {
    if (!newPersonName.trim()) return;

    setIsLoading(true);
    try {
      // Call MCP tool: add_person
      const response = await fetch('/api/mcp/call', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          tool: 'add_person',
          input: {
            name: newPersonName,
            email: newPersonEmail || undefined,
          },
        }),
      });
      const result = await response.json();
      setPeople([...people, result.person]);
      setNewPersonName('');
      setNewPersonEmail('');
      setActiveTab('search');
    } catch (error) {
      console.error('Error adding person:', error);
    } finally {
      setIsLoading(false);
    }
  }, [newPersonName, newPersonEmail, people]);

  const strengthColor = (label: string) => {
    switch (label) {
      case 'ACTIVE':
        return 'bg-green-100 text-green-800';
      case 'WARM':
        return 'bg-yellow-100 text-yellow-800';
      case 'FADING':
        return 'bg-red-100 text-red-800';
      default:
        return 'bg-gray-100 text-gray-800';
    }
  };

  const strengthIcon = (label: string) => {
    switch (label) {
      case 'ACTIVE':
        return <TrendingUp className="w-4 h-4" />;
      case 'FADING':
        return <TrendingDown className="w-4 h-4" />;
      default:
        return null;
    }
  };

  const daysAgo = (date: string) => {
    const days = Math.floor((Date.now() - new Date(date).getTime()) / (1000 * 60 * 60 * 24));
    if (days === 0) return 'Today';
    if (days === 1) return 'Yesterday';
    return `${days} days ago`;
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 to-slate-100 p-6">
      <div className="max-w-6xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-4xl font-bold text-slate-900 mb-2">Hermes Rolodex™</h1>
          <p className="text-slate-600">Intelligent relationship management with strength decay and fuzzy recall</p>
        </div>

        {/* Tab Navigation */}
        <div className="flex gap-4 mb-8 border-b border-slate-200">
          {(['search', 'add', 'upcoming'] as const).map((tab) => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              className={`px-4 py-3 font-medium transition-colors ${
                activeTab === tab
                  ? 'text-blue-600 border-b-2 border-blue-600'
                  : 'text-slate-600 hover:text-slate-900'
              }`}
            >
              {tab === 'search' && <Search className="inline w-4 h-4 mr-2" />}
              {tab === 'add' && <Plus className="inline w-4 h-4 mr-2" />}
              {tab === 'upcoming' && <Calendar className="inline w-4 h-4 mr-2" />}
              {tab.charAt(0).toUpperCase() + tab.slice(1)}
            </button>
          ))}
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Main Content */}
          <div className="lg:col-span-2">
            {activeTab === 'search' && (
              <div className="space-y-6">
                {/* Search Box */}
                <div className="relative">
                  <Search className="absolute left-4 top-3 w-5 h-5 text-slate-400" />
                  <input
                    type="text"
                    placeholder="Search for a person... (fuzzy recall enabled)"
                    value={searchQuery}
                    onChange={(e) => {
                      setSearchQuery(e.target.value);
                      handleSearch(e.target.value);
                    }}
                    className="w-full pl-12 pr-4 py-3 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>

                {/* Results */}
                <div className="space-y-3">
                  {isLoading ? (
                    <div className="text-center py-8">
                      <div className="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    </div>
                  ) : people.length > 0 ? (
                    people.map((person) => (
                      <div
                        key={person.id}
                        onClick={() => handleSelectPerson(person)}
                        className={`p-4 rounded-lg cursor-pointer transition-colors ${
                          selectedPerson?.id === person.id
                            ? 'bg-blue-50 border-2 border-blue-500'
                            : 'bg-white border border-slate-200 hover:bg-slate-50'
                        }`}
                      >
                        <div className="flex items-start justify-between">
                          <div className="flex-1">
                            <h3 className="font-semibold text-slate-900">{person.name}</h3>
                            {person.email && (
                              <p className="text-sm text-slate-500 flex items-center mt-1">
                                <Mail className="w-4 h-4 mr-2" />
                                {person.email}
                              </p>
                            )}
                            <p className="text-sm text-slate-500 mt-1">
                              Last contact: {daysAgo(person.last_contact_at)}
                            </p>
                          </div>
                          <div className={`px-3 py-1 rounded-full text-sm font-medium flex items-center gap-2 ${strengthColor(person.strength_label)}`}>
                            {strengthIcon(person.strength_label)}
                            {person.strength_label}
                          </div>
                        </div>
                        <div className="mt-2 w-full bg-slate-200 rounded-full h-2">
                          <div
                            className="bg-blue-600 h-2 rounded-full transition-all"
                            style={{ width: `${Math.max(person.strength * 100, 5)}%` }}
                          ></div>
                        </div>
                      </div>
                    ))
                  ) : searchQuery ? (
                    <div className="text-center py-8 text-slate-500">
                      <p>No results found. Try a different search or add a new person.</p>
                    </div>
                  ) : null}
                </div>
              </div>
            )}

            {activeTab === 'add' && (
              <div className="bg-white rounded-lg p-6 border border-slate-200">
                <h2 className="text-xl font-semibold text-slate-900 mb-6">Add New Person</h2>
                <div className="space-y-4">
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-2">Name *</label>
                    <input
                      type="text"
                      value={newPersonName}
                      onChange={(e) => setNewPersonName(e.target.value)}
                      placeholder="John Doe"
                      className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-slate-700 mb-2">Email</label>
                    <input
                      type="email"
                      value={newPersonEmail}
                      onChange={(e) => setNewPersonEmail(e.target.value)}
                      placeholder="john@example.com"
                      className="w-full px-4 py-2 border border-slate-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  </div>
                  <button
                    onClick={handleAddPerson}
                    disabled={!newPersonName.trim() || isLoading}
                    className="w-full bg-blue-600 text-white font-medium py-2 rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                  >
                    {isLoading ? 'Adding...' : 'Add Person'}
                  </button>
                </div>
              </div>
            )}

            {activeTab === 'upcoming' && (
              <div className="bg-white rounded-lg p-6 border border-slate-200">
                <h2 className="text-xl font-semibold text-slate-900 mb-6">Upcoming Events & Reminders</h2>
                <p className="text-slate-600">Configure reminders and upcoming milestones in your Hermes configuration.</p>
              </div>
            )}
          </div>

          {/* Sidebar - Person Details */}
          {selectedPerson && (
            <div className="bg-white rounded-lg p-6 border border-slate-200 sticky top-6">
              <h2 className="text-xl font-semibold text-slate-900 mb-6">{selectedPerson.name}</h2>

              <div className="space-y-4 mb-6">
                <div className={`inline-flex items-center gap-2 px-3 py-1 rounded-full font-medium ${strengthColor(selectedPerson.strength_label)}`}>
                  {strengthIcon(selectedPerson.strength_label)}
                  {selectedPerson.strength_label}
                </div>

                {selectedPerson.email && (
                  <div className="flex items-center gap-3 text-slate-600">
                    <Mail className="w-4 h-4" />
                    <a href={`mailto:${selectedPerson.email}`} className="text-blue-600 hover:underline">
                      {selectedPerson.email}
                    </a>
                  </div>
                )}

                <div className="flex items-center gap-3 text-slate-600">
                  <Calendar className="w-4 h-4" />
                  <span>Last contact {daysAgo(selectedPerson.last_contact_at)}</span>
                </div>

                <div className="pt-4 border-t">
                  <p className="text-sm text-slate-600 mb-2">Relationship Strength</p>
                  <div className="w-full bg-slate-200 rounded-full h-3">
                    <div
                      className="bg-gradient-to-r from-red-500 to-green-500 h-3 rounded-full transition-all"
                      style={{ width: `${Math.max(selectedPerson.strength * 100, 5)}%` }}
                    ></div>
                  </div>
                  <p className="text-xs text-slate-500 mt-1">{(selectedPerson.strength * 100).toFixed(1)}% strength</p>
                </div>
              </div>

              <div className="space-y-2">
                <button className="w-full flex items-center justify-center gap-2 bg-blue-50 text-blue-600 hover:bg-blue-100 py-2 rounded-lg transition-colors font-medium">
                  <Mail className="w-4 h-4" />
                  Draft Outreach
                </button>
                <button className="w-full flex items-center justify-center gap-2 bg-green-50 text-green-600 hover:bg-green-100 py-2 rounded-lg transition-colors font-medium">
                  <Calendar className="w-4 h-4" />
                  Set Reminder
                </button>
                <button className="w-full flex items-center justify-center gap-2 bg-purple-50 text-purple-600 hover:bg-purple-100 py-2 rounded-lg transition-colors font-medium">
                  <LinkIcon className="w-4 h-4" />
                  View Graph
                </button>
              </div>

              {memories.length > 0 && (
                <div className="mt-6 pt-6 border-t">
                  <h3 className="font-semibold text-slate-900 mb-3 flex items-center gap-2">
                    <MessageCircle className="w-4 h-4" />
                    Memories ({memories.length})
                  </h3>
                  <div className="space-y-3 max-h-40 overflow-y-auto">
                    {memories.map((memory) => (
                      <div key={memory.id} className="p-2 bg-slate-50 rounded text-sm text-slate-600">
                        <p>{memory.content}</p>
                        <p className="text-xs text-slate-500 mt-1">{daysAgo(memory.created_at)}</p>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default HermesRolodexUI;
