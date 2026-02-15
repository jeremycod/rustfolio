import { useState } from 'react';
import {
  Box,
  Paper,
  Typography,
  TextField,
  Button,
  Alert,
  Chip,
  CircularProgress,
  Divider,
} from '@mui/material';
import { Psychology, Send, Info, CheckCircle, Warning, Error as ErrorIcon } from '@mui/icons-material';
import { useMutation } from '@tanstack/react-query';
import { askPortfolioQuestion } from '../lib/endpoints';
import AIBadge from './AIBadge';
import type { QAConversation, Confidence } from '../types';

type Props = {
  portfolioId: string;
};

export default function PortfolioQA({ portfolioId }: Props) {
  const [question, setQuestion] = useState('');
  const [conversations, setConversations] = useState<QAConversation[]>([]);

  const askMutation = useMutation({
    mutationFn: (q: string) => askPortfolioQuestion(portfolioId, { question: q }),
    onSuccess: (answer) => {
      setConversations([...conversations, { question: { question }, answer }]);
      setQuestion('');
    },
  });

  const handleSubmit = () => {
    if (question.trim() && !askMutation.isPending) {
      askMutation.mutate(question.trim());
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  const handleFollowUpClick = (followUpQuestion: string) => {
    setQuestion(followUpQuestion);
  };

  const getConfidenceIcon = (confidence: Confidence) => {
    switch (confidence) {
      case 'high':
        return <CheckCircle fontSize="small" color="success" />;
      case 'medium':
        return <Warning fontSize="small" color="warning" />;
      case 'low':
        return <ErrorIcon fontSize="small" color="error" />;
    }
  };

  const getConfidenceColor = (confidence: Confidence) => {
    switch (confidence) {
      case 'high':
        return 'success';
      case 'medium':
        return 'warning';
      case 'low':
        return 'error';
    }
  };

  return (
    <Paper sx={{ p: 3 }}>
      {/* Header */}
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 2 }}>
        <Psychology color="primary" />
        <Typography variant="h6">Portfolio Assistant</Typography>
        <AIBadge />
      </Box>

      <Alert severity="info" sx={{ mb: 3 }} icon={<Info />}>
        Ask questions about your portfolio. The AI assistant will provide educational insights based on your portfolio data.
      </Alert>

      {/* Conversation History */}
      {conversations.length > 0 && (
        <Box sx={{ mb: 3, maxHeight: '400px', overflowY: 'auto' }}>
          {conversations.map((convo, idx) => (
            <Box key={idx} sx={{ mb: 3 }}>
              {/* User Question */}
              <Box sx={{ display: 'flex', justifyContent: 'flex-end', mb: 1 }}>
                <Paper
                  elevation={1}
                  sx={{
                    p: 2,
                    bgcolor: 'primary.main',
                    color: 'primary.contrastText',
                    maxWidth: '70%',
                    borderRadius: 2,
                  }}
                >
                  <Typography variant="body1">{convo.question.question}</Typography>
                </Paper>
              </Box>

              {/* AI Answer */}
              <Box sx={{ display: 'flex', justifyContent: 'flex-start', mb: 1 }}>
                <Paper
                  elevation={1}
                  sx={{
                    p: 2,
                    bgcolor: 'background.default',
                    maxWidth: '70%',
                    borderRadius: 2,
                  }}
                >
                  <Typography variant="body1" sx={{ mb: 1 }}>
                    {convo.answer.answer}
                  </Typography>

                  {/* Metadata */}
                  <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap', mt: 2 }}>
                    <Chip
                      icon={getConfidenceIcon(convo.answer.confidence)}
                      label={`${convo.answer.confidence} confidence`}
                      size="small"
                      color={getConfidenceColor(convo.answer.confidence)}
                      variant="outlined"
                    />
                    {convo.answer.sources.map((source, sidx) => (
                      <Chip key={sidx} label={source} size="small" variant="outlined" />
                    ))}
                  </Box>

                  {/* Follow-up Questions */}
                  {convo.answer.follow_up_questions.length > 0 && (
                    <Box sx={{ mt: 2 }}>
                      <Typography variant="caption" color="text.secondary" display="block" sx={{ mb: 1 }}>
                        Follow-up questions:
                      </Typography>
                      <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
                        {convo.answer.follow_up_questions.map((fq, fqidx) => (
                          <Chip
                            key={fqidx}
                            label={fq}
                            size="small"
                            onClick={() => handleFollowUpClick(fq)}
                            sx={{ cursor: 'pointer' }}
                          />
                        ))}
                      </Box>
                    </Box>
                  )}

                  <Typography variant="caption" color="text.secondary" display="block" sx={{ mt: 1 }}>
                    {new Date(convo.answer.generated_at).toLocaleTimeString()}
                  </Typography>
                </Paper>
              </Box>
            </Box>
          ))}
        </Box>
      )}

      {/* Loading State */}
      {askMutation.isPending && (
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2, p: 2, bgcolor: 'action.hover', borderRadius: 1 }}>
          <CircularProgress size={20} />
          <Typography variant="body2" color="text.secondary">
            Thinking...
          </Typography>
        </Box>
      )}

      {/* Error State */}
      {askMutation.isError && (
        <Alert severity="error" sx={{ mb: 2 }}>
          {askMutation.error instanceof Error
            ? askMutation.error.message
            : 'Failed to get an answer. Please try again.'}
        </Alert>
      )}

      <Divider sx={{ my: 2 }} />

      {/* Input Area */}
      <Box sx={{ display: 'flex', gap: 1 }}>
        <TextField
          fullWidth
          multiline
          maxRows={3}
          placeholder="Ask a question about your portfolio..."
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          onKeyPress={handleKeyPress}
          disabled={askMutation.isPending}
          size="small"
        />
        <Button
          variant="contained"
          endIcon={<Send />}
          onClick={handleSubmit}
          disabled={!question.trim() || askMutation.isPending}
          sx={{ minWidth: '100px' }}
        >
          Ask
        </Button>
      </Box>

      {/* Suggested Questions */}
      {conversations.length === 0 && (
        <Box sx={{ mt: 2 }}>
          <Typography variant="caption" color="text.secondary" display="block" sx={{ mb: 1 }}>
            Try asking:
          </Typography>
          <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 0.5 }}>
            {[
              'What are my top 3 holdings?',
              'How is my portfolio performing?',
              'What are my biggest risks?',
              'How diversified is my portfolio?',
            ].map((suggestion) => (
              <Chip
                key={suggestion}
                label={suggestion}
                size="small"
                onClick={() => setQuestion(suggestion)}
                sx={{ cursor: 'pointer' }}
              />
            ))}
          </Box>
        </Box>
      )}
    </Paper>
  );
}
