/*
Copyright (C) 1996-2001 Id Software, Inc.
Copyright (C) 2010-2014 QuakeSpasm developers

This program is free software; you can redistribute it and/or
modify it under the terms of the GNU General Public License
as published by the Free Software Foundation; either version 2
of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.

See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program; if not, write to the Free Software
Foundation, Inc., 59 Temple Place - Suite 330, Boston, MA  02111-1307, USA.

*/

#include "quakedef.h"

static const char *pr_opnames[] = {"DONE",

								   "MUL_F",	   "MUL_V",	   "MUL_FV",   "MUL_VF",

								   "DIV",

								   "ADD_F",	   "ADD_V",

								   "SUB_F",	   "SUB_V",

								   "EQ_F",	   "EQ_V",	   "EQ_S",	   "EQ_E",		 "EQ_FNC",

								   "NE_F",	   "NE_V",	   "NE_S",	   "NE_E",		 "NE_FNC",

								   "LE",	   "GE",	   "LT",	   "GT",

								   "INDIRECT", "INDIRECT", "INDIRECT", "INDIRECT",	 "INDIRECT",   "INDIRECT",

								   "ADDRESS",

								   "STORE_F",  "STORE_V",  "STORE_S",  "STORE_ENT",	 "STORE_FLD",  "STORE_FNC",

								   "STOREP_F", "STOREP_V", "STOREP_S", "STOREP_ENT", "STOREP_FLD", "STOREP_FNC",

								   "RETURN",

								   "NOT_F",	   "NOT_V",	   "NOT_S",	   "NOT_ENT",	 "NOT_FNC",

								   "IF",	   "IFNOT",

								   "CALL0",	   "CALL1",	   "CALL2",	   "CALL3",		 "CALL4",	   "CALL5",		 "CALL6", "CALL7", "CALL8",

								   "STATE",

								   "GOTO",

								   "AND",	   "OR",

								   "BITAND",   "BITOR"};

const char *PR_GlobalString (int ofs);
const char *PR_GlobalStringNoContents (int ofs);

//=============================================================================

/*
=================
PR_PrintStatement
=================
*/
static void PR_PrintStatement (dstatement_t *s)
{
	int i;

	if ((unsigned int)s->op < countof (pr_opnames))
	{
		Con_Printf ("%s ", pr_opnames[s->op]);
		i = strlen (pr_opnames[s->op]);
		for (; i < 10; i++)
			Con_Printf (" ");
	}

	if (s->op == OP_IF || s->op == OP_IFNOT)
		Con_Printf ("%sbranch %i", PR_GlobalString (s->a), s->b);
	else if (s->op == OP_GOTO)
	{
		Con_Printf ("branch %i", s->a);
	}
	else if ((unsigned int)(s->op - OP_STORE_F) < 6)
	{
		Con_Printf ("%s", PR_GlobalString (s->a));
		Con_Printf ("%s", PR_GlobalStringNoContents (s->b));
	}
	else
	{
		if (s->a)
			Con_Printf ("%s", PR_GlobalString (s->a));
		if (s->b)
			Con_Printf ("%s", PR_GlobalString (s->b));
		if (s->c)
			Con_Printf ("%s", PR_GlobalStringNoContents (s->c));
	}
	Con_Printf ("\n");
}

/*
============
PR_StackTrace
============
*/
static void PR_StackTrace (void)
{
	int			 i;
	dfunction_t *f;

	if (qcvm->depth == 0)
	{
		Con_Printf ("<NO STACK>\n");
		return;
	}

	qcvm->stack[qcvm->depth].f = qcvm->xfunction;
	for (i = qcvm->depth; i >= 0; i--)
	{
		f = qcvm->stack[i].f;
		if (!f)
		{
			Con_Printf ("<NO FUNCTION>\n");
		}
		else
		{
			Con_Printf ("%12s : %s\n", PR_GetString (f->s_file), PR_GetString (f->s_name));
		}
	}
}

/*
============
PR_Profile_f

============
*/
void PR_Profile_f (void)
{
	int			 i, num;
	int			 pmax;
	dfunction_t *f, *best;

	if (!sv.active)
		return;

	PR_SwitchQCVM (&sv.qcvm);

	num = 0;
	do
	{
		pmax = 0;
		best = NULL;
		for (i = 0; i < qcvm->progs->numfunctions; i++)
		{
			f = &qcvm->functions[i];
			if (f->profile > pmax)
			{
				pmax = f->profile;
				best = f;
			}
		}
		if (best)
		{
			if (num < 10)
				Con_Printf ("%7i %s\n", best->profile, PR_GetString (best->s_name));
			num++;
			best->profile = 0;
		}
	} while (best);

	PR_SwitchQCVM (NULL);
}

/*
============
PR_RunError

Aborts the currently executing function
============
*/
void PR_RunError (const char *error, ...)
{
	va_list argptr;
	char	string[1024];

	va_start (argptr, error);
	q_vsnprintf (string, sizeof (string), error, argptr);
	va_end (argptr);

	PR_PrintStatement (qcvm->statements + qcvm->xstatement);
	PR_StackTrace ();

	Con_Printf ("%s\n", string);

	qcvm->depth = 0; // dump the stack so host_error can shutdown functions

	Host_Error ("Program error");
}

void PR_RunWarning (const char *error, ...)
{
	va_list argptr;
	char	string[1024];

	va_start (argptr, error);
	q_vsnprintf (string, sizeof (string), error, argptr);
	va_end (argptr);

	PR_PrintStatement (qcvm->statements + qcvm->xstatement);
	PR_StackTrace ();

	Con_Warning ("%s\n", string);
}

/*
====================
PR_EnterFunction

Returns the new program statement counter
====================
*/
static int PR_EnterFunction (dfunction_t *f)
{
	int i, j, c, o;

	qcvm->stack[qcvm->depth].s = qcvm->xstatement;
	qcvm->stack[qcvm->depth].f = qcvm->xfunction;
	qcvm->depth++;
	if (qcvm->depth >= MAX_STACK_DEPTH)
		PR_RunError ("stack overflow");

	// save off any locals that the new function steps on
	c = f->locals;
	if (qcvm->localstack_used + c > LOCALSTACK_SIZE)
		PR_RunError ("PR_ExecuteProgram: locals stack overflow");

	for (i = 0; i < c; i++)
		qcvm->localstack[qcvm->localstack_used + i] = ((int *)qcvm->globals)[f->parm_start + i];
	qcvm->localstack_used += c;

	// copy parameters
	o = f->parm_start;
	for (i = 0; i < f->numparms; i++)
	{
		for (j = 0; j < f->parm_size[i]; j++)
		{
			((int *)qcvm->globals)[o] = ((int *)qcvm->globals)[OFS_PARM0 + i * 3 + j];
			o++;
		}
	}

	qcvm->xfunction = f;
	return f->first_statement - 1; // offset the s++
}

/*
====================
PR_LeaveFunction
====================
*/
static int PR_LeaveFunction (void)
{
	int i, c;

	if (qcvm->depth <= 0)
		Host_Error ("prog stack underflow");

	// Restore locals from the stack
	c = qcvm->xfunction->locals;
	qcvm->localstack_used -= c;
	if (qcvm->localstack_used < 0)
		PR_RunError ("PR_ExecuteProgram: locals stack underflow");

	for (i = 0; i < c; i++)
		((int *)qcvm->globals)[qcvm->xfunction->parm_start + i] = qcvm->localstack[qcvm->localstack_used + i];

	// up stack
	qcvm->depth--;
	qcvm->xfunction = qcvm->stack[qcvm->depth].f;
	return qcvm->stack[qcvm->depth].s;
}

/*
====================
PR_ExecuteProgram

The interpretation main loop
====================
*/
#define OPA ((eval_t *)&qcvm->globals[(unsigned short)st->a])
#define OPB ((eval_t *)&qcvm->globals[(unsigned short)st->b])
#define OPC ((eval_t *)&qcvm->globals[(unsigned short)st->c])

void PR_ExecuteProgram (func_t fnum)
{
	eval_t		 *ptr;
	dstatement_t *st;
	dfunction_t	 *f, *newf;
	int			  profile, startprofile;
	edict_t		 *ed;
	int			  exitdepth;

	if (!fnum || fnum >= (func_t)qcvm->progs->numfunctions)
	{
		if (pr_global_struct->self)
			ED_Print (PROG_TO_EDICT (pr_global_struct->self));
		Host_Error ("PR_ExecuteProgram: NULL function");
	}

	f = &qcvm->functions[fnum];

	// FIXME: if this is a builtin, then we're going to crash.

	qcvm->trace = false;

	// make a stack frame
	exitdepth = qcvm->depth;

	st = &qcvm->statements[PR_EnterFunction (f)];
	startprofile = profile = 0;

	while (1)
	{
		st++; /* next statement */

		if (++profile > 0x1000000) // spike -- was decimal 100000, 0x10000000 in QSS
		{
			qcvm->xstatement = st - qcvm->statements;
			PR_RunError ("runaway loop error");
		}

		if (qcvm->trace)
			PR_PrintStatement (st);

		switch (st->op)
		{
		case OP_ADD_F:
			OPC->_float = OPA->_float + OPB->_float;
			break;
		case OP_ADD_V:
			OPC->vector[0] = OPA->vector[0] + OPB->vector[0];
			OPC->vector[1] = OPA->vector[1] + OPB->vector[1];
			OPC->vector[2] = OPA->vector[2] + OPB->vector[2];
			break;

		case OP_SUB_F:
			OPC->_float = OPA->_float - OPB->_float;
			break;
		case OP_SUB_V:
			OPC->vector[0] = OPA->vector[0] - OPB->vector[0];
			OPC->vector[1] = OPA->vector[1] - OPB->vector[1];
			OPC->vector[2] = OPA->vector[2] - OPB->vector[2];
			break;

		case OP_MUL_F:
			OPC->_float = OPA->_float * OPB->_float;
			break;
		case OP_MUL_V:
			OPC->_float = OPA->vector[0] * OPB->vector[0] + OPA->vector[1] * OPB->vector[1] + OPA->vector[2] * OPB->vector[2];
			break;
		case OP_MUL_FV:
			OPC->vector[0] = OPA->_float * OPB->vector[0];
			OPC->vector[1] = OPA->_float * OPB->vector[1];
			OPC->vector[2] = OPA->_float * OPB->vector[2];
			break;
		case OP_MUL_VF:
			OPC->vector[0] = OPB->_float * OPA->vector[0];
			OPC->vector[1] = OPB->_float * OPA->vector[1];
			OPC->vector[2] = OPB->_float * OPA->vector[2];
			break;

		case OP_DIV_F:
			OPC->_float = OPA->_float / OPB->_float;
			break;

		case OP_BITAND:
			OPC->_float = (int)OPA->_float & (int)OPB->_float;
			break;

		case OP_BITOR:
			OPC->_float = (int)OPA->_float | (int)OPB->_float;
			break;

		case OP_GE:
			OPC->_float = OPA->_float >= OPB->_float;
			break;
		case OP_LE:
			OPC->_float = OPA->_float <= OPB->_float;
			break;
		case OP_GT:
			OPC->_float = OPA->_float > OPB->_float;
			break;
		case OP_LT:
			OPC->_float = OPA->_float < OPB->_float;
			break;
		case OP_AND:
			OPC->_float = OPA->_float && OPB->_float;
			break;
		case OP_OR:
			OPC->_float = OPA->_float || OPB->_float;
			break;

		case OP_NOT_F:
			OPC->_float = !OPA->_float;
			break;
		case OP_NOT_V:
			OPC->_float = !OPA->vector[0] && !OPA->vector[1] && !OPA->vector[2];
			break;
		case OP_NOT_S:
			OPC->_float = !OPA->string || !*PR_GetString (OPA->string);
			break;
		case OP_NOT_FNC:
			OPC->_float = !OPA->function;
			break;
		case OP_NOT_ENT:
			OPC->_float = (PROG_TO_EDICT (OPA->edict) == qcvm->edicts);
			break;

		case OP_EQ_F:
			OPC->_float = OPA->_float == OPB->_float;
			break;
		case OP_EQ_V:
			OPC->_float = (OPA->vector[0] == OPB->vector[0]) && (OPA->vector[1] == OPB->vector[1]) && (OPA->vector[2] == OPB->vector[2]);
			break;
		case OP_EQ_S:
			OPC->_float = !strcmp (PR_GetString (OPA->string), PR_GetString (OPB->string));
			break;
		case OP_EQ_E:
			OPC->_float = OPA->_int == OPB->_int;
			break;
		case OP_EQ_FNC:
			OPC->_float = OPA->function == OPB->function;
			break;

		case OP_NE_F:
			OPC->_float = OPA->_float != OPB->_float;
			break;
		case OP_NE_V:
			OPC->_float = (OPA->vector[0] != OPB->vector[0]) || (OPA->vector[1] != OPB->vector[1]) || (OPA->vector[2] != OPB->vector[2]);
			break;
		case OP_NE_S:
			OPC->_float = strcmp (PR_GetString (OPA->string), PR_GetString (OPB->string));
			break;
		case OP_NE_E:
			OPC->_float = OPA->_int != OPB->_int;
			break;
		case OP_NE_FNC:
			OPC->_float = OPA->function != OPB->function;
			break;

		case OP_STORE_F:
		case OP_STORE_ENT:
		case OP_STORE_FLD: // integers
		case OP_STORE_S:
		case OP_STORE_FNC: // pointers
			OPB->_int = OPA->_int;
			break;
		case OP_STORE_V:
			OPB->vector[0] = OPA->vector[0];
			OPB->vector[1] = OPA->vector[1];
			OPB->vector[2] = OPA->vector[2];
			break;

		case OP_STOREP_F:
		case OP_STOREP_ENT:
		case OP_STOREP_FLD: // integers
		case OP_STOREP_S:
		case OP_STOREP_FNC: // pointers
			ptr = (eval_t *)((byte *)qcvm->edicts + OPB->_int);
			ptr->_int = OPA->_int;
			break;
		case OP_STOREP_V:
			ptr = (eval_t *)((byte *)qcvm->edicts + OPB->_int);
			ptr->vector[0] = OPA->vector[0];
			ptr->vector[1] = OPA->vector[1];
			ptr->vector[2] = OPA->vector[2];
			break;

		case OP_ADDRESS:
			ed = PROG_TO_EDICT (OPA->edict);
			NUM_FOR_EDICT (ed); // Make sure it's in range
			if (ed == (edict_t *)qcvm->edicts && sv.state == ss_active)
			{
				qcvm->xstatement = st - qcvm->statements;
				PR_RunError ("assignment to world entity");
			}
			OPC->_int = (byte *)((int *)&ed->v + OPB->_int) - (byte *)qcvm->edicts;
			break;

		case OP_LOAD_F:
		case OP_LOAD_FLD:
		case OP_LOAD_ENT:
		case OP_LOAD_S:
		case OP_LOAD_FNC:
			ed = PROG_TO_EDICT (OPA->edict);
			NUM_FOR_EDICT (ed); // Make sure it's in range
			OPC->_int = ((eval_t *)((int *)&ed->v + OPB->_int))->_int;
			break;

		case OP_LOAD_V:
			ed = PROG_TO_EDICT (OPA->edict);
			NUM_FOR_EDICT (ed); // Make sure it's in range
			ptr = (eval_t *)((int *)&ed->v + OPB->_int);
			OPC->vector[0] = ptr->vector[0];
			OPC->vector[1] = ptr->vector[1];
			OPC->vector[2] = ptr->vector[2];
			break;

		case OP_IFNOT:
			if (!OPA->_int)
				st += st->b - 1; /* -1 to offset the st++ */
			break;

		case OP_IF:
			if (OPA->_int)
				st += st->b - 1; /* -1 to offset the st++ */
			break;

		case OP_GOTO:
			st += st->a - 1; /* -1 to offset the st++ */
			break;

		case OP_CALL0:
		case OP_CALL1:
		case OP_CALL2:
		case OP_CALL3:
		case OP_CALL4:
		case OP_CALL5:
		case OP_CALL6:
		case OP_CALL7:
		case OP_CALL8:
			qcvm->xfunction->profile += profile - startprofile;
			startprofile = profile;
			qcvm->xstatement = st - qcvm->statements;
			qcvm->argc = st->op - OP_CALL0;
			if (!OPA->function)
				PR_RunError ("NULL function");
			newf = &qcvm->functions[OPA->function];
			if (newf->first_statement < 0)
			{ // Built-in function
				int i = -newf->first_statement;
				if (i >= qcvm->numbuiltins)
					i = 0; // just invoke the fixme builtin.
				qcvm->builtins[i]();
				break;
			}
			// Normal function
			st = &qcvm->statements[PR_EnterFunction (newf)];
			break;

		case OP_DONE:
		case OP_RETURN:
			qcvm->xfunction->profile += profile - startprofile;
			startprofile = profile;
			qcvm->xstatement = st - qcvm->statements;
			qcvm->globals[OFS_RETURN] = qcvm->globals[(unsigned short)st->a];
			qcvm->globals[OFS_RETURN + 1] = qcvm->globals[(unsigned short)st->a + 1];
			qcvm->globals[OFS_RETURN + 2] = qcvm->globals[(unsigned short)st->a + 2];
			st = &qcvm->statements[PR_LeaveFunction ()];
			if (qcvm->depth == exitdepth)
			{ // Done
				goto PR_ExecuteProgram_end;
			}
			break;

		case OP_STATE:
			ed = PROG_TO_EDICT (pr_global_struct->self);
			ed->v.nextthink = pr_global_struct->time + 0.1;
			ed->v.frame = OPA->_float;
			ed->v.think = OPB->function;
			break;

		default:
			qcvm->xstatement = st - qcvm->statements;
			PR_RunError ("Bad opcode %i", st->op);
		}
	} /* end of while(1) loop */

PR_ExecuteProgram_end:
	if (qcvm == &sv.qcvm)
	{
		if (!sv.reached_intermission)
		{
			// TODO check that a svc_intermission packet was sent instead of checking these conditions.
			qboolean some_players_checked = false;
			qboolean all_players_in_intermission = true;
			for (int i = 0; i < svs.maxclients; i++)
			{
				if (!svs.clients[i].active)
					continue;
				some_players_checked = true;
				edict_t *edict = svs.clients[i].edict;
				if (edict->v.takedamage != DAMAGE_NO || edict->v.solid != SOLID_NOT || edict->v.movetype != MOVETYPE_NONE)
				{
					all_players_in_intermission = false;
					break;
				}
			}

			if (some_players_checked && all_players_in_intermission)
			{
				// TODO don't do this lookup on every run
				// TODO do this lookup first instead of players check
				ddef_t *intermission_running = ED_FindGlobal ("intermission_running");
				if (intermission_running && qcvm->globals[intermission_running->ofs] > 0)
				{
					sv.reached_intermission = true;

					const int total_secrets = pr_global_struct->total_secrets;
					uint8_t	 *secrets_not_found = calloc (total_secrets, sizeof (uint8_t));
					int		  secrets_not_found_count = 0;

					edict_t *ent = NEXT_EDICT (qcvm->edicts);
					for (int e = 1; e < qcvm->num_edicts; e++, ent = NEXT_EDICT (ent))
					{
						if (ent->free)
							continue;
						// Arcane Dimensions keeps secret triggers around but them not solid.
						if (ent->v.solid == SOLID_NOT)
							continue;
						if (ent->secret_index_plus_one != 0)
						{
							int secret_index = ent->secret_index_plus_one - 1;
							if (secret_index < total_secrets)
							{
								secrets_not_found[secret_index] = 1;
								secrets_not_found_count++;
							}
						}
					}

					const int expected_secrets_not_found = pr_global_struct->total_secrets - pr_global_struct->found_secrets;
					if (secrets_not_found_count != expected_secrets_not_found)
					{
						Con_DPrintf ("Warning: only identified %d out of %d secrets not found.\n", secrets_not_found_count, expected_secrets_not_found);
					}

					const int found_secrets = pr_global_struct->found_secrets;
					int		  identified_secrets = 0;
					uint16_t *secrets = malloc (found_secrets * sizeof (uint16_t));

					for (int i = 0; i < total_secrets && identified_secrets < found_secrets; i++)
					{
						if (secrets_not_found[i] == 0)
						{
							secrets[identified_secrets++] = i;
						}
					}

					MSG_WriteByte (&sv.multicast, svcmx_level_complete);
					MSG_WriteShort (&sv.multicast, current_skill);
					MSG_WriteShort (&sv.multicast, identified_secrets);
					SZ_Write (&sv.multicast, secrets, identified_secrets * sizeof (uint16_t));
					// TODO send only to clients with correct protocol flags
					SV_Multicast (MULTICAST_ALL_R, NULL, 0, 0);

					free (secrets_not_found);
					free (secrets);
				}
			}
		}
	}
}
#undef OPA
#undef OPB
#undef OPC
